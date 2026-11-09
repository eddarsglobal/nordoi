//! G0.3 R4L: independently signed XPC clients; authenticated positive
//! reachability control and negative side-effect witness in real launchd service.
//! Research only. No production trust anchor, no arbitrary host IO authority.
#![allow(dead_code)]

#[path = "../src/resource_task_r02.rs"]
mod reference;
use reference::{Model, ModelError, ResourceEffect, ScopeBudget};

#[cfg(target_os = "macos")]
mod native {
    use std::env;
    use std::fs::{self, OpenOptions};
    use std::io::{Read, Write};
    use std::path::{Path, PathBuf};
    use std::process::{Child, Command, Stdio};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::thread;
    use std::time::{Duration, Instant};

    const LIMIT: u64 = 4096;
    static SERIAL: AtomicU64 = AtomicU64::new(1);
    const SERVICE: &str = r#"
#include <xpc/xpc.h>
#include <dispatch/dispatch.h>
#include <stdatomic.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include <unistd.h>
static _Atomic unsigned long long accepted = 0;
static const char *CLIENT_REQ = "identifier \"org.eddarsglobal.nordoi.r4l.client\"";
static void accept_peer(xpc_connection_t peer) {
#ifndef R4L_OPEN_CONTROL
    int status=xpc_connection_set_peer_code_signing_requirement(peer, CLIENT_REQ);
    if(status!=0) { xpc_connection_cancel(peer); return; }
#endif
    xpc_connection_set_event_handler(peer, ^(xpc_object_t msg) {
        if(xpc_get_type(msg)!=XPC_TYPE_DICTIONARY) return;
        xpc_object_t reply=xpc_dictionary_create_reply(msg);
        if(!reply) return;
        const char *op=xpc_dictionary_get_string(msg,"op");
        if(op && strcmp(op,"count")==0 &&
           xpc_dictionary_get_count(msg)==1) {
            xpc_dictionary_set_string(reply,"status","counted");
            xpc_dictionary_set_uint64(reply,"accepted",
                atomic_load_explicit(&accepted,memory_order_relaxed));
            xpc_dictionary_set_int64(reply,"pid",(int64_t)getpid());
        } else if(op && strcmp(op,"ping")==0 &&
                  xpc_dictionary_get_count(msg)==3 &&
                  xpc_dictionary_get_string(msg,"nonce") &&
                  strcmp(xpc_dictionary_get_string(msg,"nonce"),"r4l-fixed-probe")==0) {
            // Only the actual XPC handler writes a test-only marker, via a
            // precise disposable descriptor the initiating peer passed.
            int fd=xpc_dictionary_dup_fd(msg,"receipt_fd");
            if(fd<0) {
                xpc_dictionary_set_string(reply,"status","denied");
            } else {
                char marker[80];
                int count=snprintf(marker,sizeof(marker),
                    "R4L|HANDLER|PID=%ld\n",(long)getpid());
                ssize_t written=(count>0 && count<(int)sizeof(marker))
                    ? write(fd,marker,(size_t)count) : -1;
                int synced=(written==(ssize_t)count) ? fsync(fd) : -1;
                close(fd);
                if(written==(ssize_t)count && synced==0) {
                    atomic_fetch_add_explicit(&accepted,1,memory_order_relaxed);
                    xpc_dictionary_set_string(reply,"status","pong");
                    xpc_dictionary_set_int64(reply,"pid",(int64_t)getpid());
                } else xpc_dictionary_set_string(reply,"status","denied");
            }
        } else {
            xpc_dictionary_set_string(reply,"status","denied");
        }
        xpc_connection_send_message(peer,reply);
        xpc_release(reply);
    });
    xpc_connection_resume(peer);
}
int main(void) { xpc_main(accept_peer); return 0; }
"#;
    const PEER: &str = r#"
#include <xpc/xpc.h>
#include <dispatch/dispatch.h>
#include <unistd.h>
#include <string.h>
#include <stdio.h>
#include <fcntl.h>
int main(int argc,char **argv) {
    if(argc!=5) return 70;
    if(strcmp(argv[3],"good")!=0 && strcmp(argv[3],"bad")!=0 &&
       strcmp(argv[3],"control")!=0) return 71;
    alarm(4);
    xpc_connection_t c=xpc_connection_create(argv[1],NULL);
    if(!c) return 72;
    xpc_connection_set_event_handler(c,^(xpc_object_t msg){ (void)msg; });
    xpc_connection_resume(c);
    xpc_object_t req=xpc_dictionary_create(NULL,NULL,0);
    xpc_dictionary_set_string(req,"op","ping");
    xpc_dictionary_set_string(req,"nonce",argv[2]);
    int fd=open(argv[4],O_WRONLY|O_APPEND|O_CLOEXEC);
    if(fd<0) return 76;
    xpc_dictionary_set_fd(req,"receipt_fd",fd);
    close(fd);
    xpc_object_t resp=xpc_connection_send_message_with_reply_sync(c,req);
    xpc_release(req);
    int result=73;
    if(resp && xpc_get_type(resp)==XPC_TYPE_DICTIONARY) {
        const char *s=xpc_dictionary_get_string(resp,"status");
        if(s && strcmp(s,"pong")==0) {
            result=(strcmp(argv[3],"bad")==0)?74:0;
        }
    } else if(resp==XPC_ERROR_PEER_CODE_SIGNING_REQUIREMENT) {
        result=(strcmp(argv[3],"bad")==0)?7:75;
    } else if(resp && xpc_get_type(resp)==XPC_TYPE_ERROR) {
        // Error categories must remain distinguishable. A generic process
        // interruption is not equivalent to a code-signing rejection.
        if(resp==XPC_ERROR_CONNECTION_INVALID) {
            fprintf(stderr,"R4L|XPC_ERROR_CONNECTION_INVALID|PEER=%s\n",argv[3]);
            result=(strcmp(argv[3],"bad")==0)?8:75;
        } else if(resp==XPC_ERROR_CONNECTION_INTERRUPTED) {
            fprintf(stderr,"R4L|XPC_ERROR_CONNECTION_INTERRUPTED|PEER=%s\n",argv[3]);
            result=(strcmp(argv[3],"bad")==0)?9:75;
        } else {
            fprintf(stderr,"R4L|UNCLASSIFIED_XPC_ERROR|PEER=%s\n",argv[3]);
            result=(strcmp(argv[3],"bad")==0)?10:75;
        }
    }
    if(resp) xpc_release(resp);
    xpc_connection_cancel(c);
    xpc_release(c);
    return result;
}
"#;
    const DRIVER: &str = r#"
#include <xpc/xpc.h>
#include <dispatch/dispatch.h>
#include <spawn.h>
#include <sys/wait.h>
#include <signal.h>
#include <sys/stat.h>
#include <unistd.h>
#include <fcntl.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
extern char **environ;
static int journal(char *path, unsigned long *first, unsigned long *second) {
    char data[256] = {0};
    int fd = open(path,O_RDONLY);
    if(fd<0) return -1;
    ssize_t n = read(fd,data,sizeof(data)-1);
    close(fd);
    if(n<0 || n>=(ssize_t)sizeof(data)-1) return -1;
    int lines=0;
    char *cursor=data;
    while(*cursor) {
        char *end=strchr(cursor,'\n');
        if(!end || end==cursor) return -1;
        *end='\0';
        unsigned long pid=0;
        char trailer='\0';
        if(sscanf(cursor,"R4L|HANDLER|PID=%lu%c",&pid,&trailer)!=1 || pid==0) return -1;
        if(lines==0 && first) *first=pid;
        if(lines==1 && second) *second=pid;
        if(++lines>3) return -1;
        cursor=end+1;
    }
    return lines;
}
static int run_peer(char *exe,char *svc,char *mode,char *journal_path) {
    pid_t pid=0;
    char *const args[]={exe,svc,"r4l-fixed-probe",mode,journal_path,NULL};
    if(posix_spawn(&pid,exe,NULL,NULL,args,environ)!=0) return -1;
    int status=0;
    if(waitpid(pid,&status,0)!=pid) return -2;
    if(WIFEXITED(status)) return WEXITSTATUS(status);
    if(WIFSIGNALED(status) && WTERMSIG(status)==SIGALRM) return 14;
    fprintf(stderr,"R4L|UNEXPECTED_PEER_SIGNAL|SIGNAL=%d\n",
            WIFSIGNALED(status)?WTERMSIG(status):-1);
    return -3;
}
int main(int argc,char **argv) {
    if(argc!=6) return 80;
    unsigned long good_pid=0,ctrl_pid=0;
    if(journal(argv[5],NULL,NULL)!=0) return 81;

    // Signed positive peer MUST reach the protected service and cause a real
    // service-origin marker in a disposable fd passed over XPC.
    int good=run_peer(argv[3],argv[1],"good",argv[5]);
    if(good!=0 || journal(argv[5],&good_pid,NULL)!=1 || good_pid==getpid()) {
        fprintf(stderr,"R4L|PROTECTED_GOOD_CONTROL_FAILED|EXIT=%d\n",good);
        return 82;
    }

    // Same maliciously identified executable MUST be able to reach the
    // identically sandboxed, separately bundled unfiltered control service.
    int bad_open=run_peer(argv[4],argv[2],"control",argv[5]);
    if(bad_open!=0 || journal(argv[5],&good_pid,&ctrl_pid)!=2
       || ctrl_pid==0 || ctrl_pid==getpid()) {
        fprintf(stderr,"R4L|BAD_PEER_OPEN_CONTROL_FAILED|EXIT=%d\n",bad_open);
        return 83;
    }

    // Differential proof: same bad executable and bundle, only the service
    // admission signing requirement differs; kernel may silently drop it.
    int bad_guarded=run_peer(argv[4],argv[1],"bad",argv[5]);
    if(bad_guarded!=14 && bad_guarded!=7 && bad_guarded!=8 && bad_guarded!=9) {
        fprintf(stderr,"R4L|BAD_GUARDED_UNEXPECTED_EXIT|EXIT=%d\n",bad_guarded);
        return 84;
    }
    if(journal(argv[5],NULL,NULL)!=2) {
        fprintf(stderr,"R4L|BAD_GUARDED_REACHED_HANDLER\n");
        return 85;
    }
    // One more guarded positive handshake AFTER the adversarial attempt
    // excludes a dead or generally unreachable protected service.
    int after_good=run_peer(argv[3],argv[1],"good",argv[5]);
    if(after_good!=0 || journal(argv[5],NULL,NULL)!=3) {
        fprintf(stderr,"R4L|GUARDED_POST_NEGATIVE_CONTROL_FAILED|EXIT=%d\n",
                after_good);
        return 86;
    }
    printf("R4L|GUARDED_GOOD=HANDLER_RECORDED|OPEN_BAD=HANDLER_RECORDED|POST_GOOD=HANDLER_RECORDED\n");
    const char *witness=(bad_guarded==14)?"SILENT_DROP":
        (bad_guarded==7)?"XPC_SIGNING_ERROR":
        (bad_guarded==8)?"CONNECTION_INVALID_DIFFERENTIAL":
        "CONNECTION_INTERRUPTED_DIFFERENTIAL";
    printf("R4L|GUARDED_BAD=NO_HANDLER_RECORD|WITNESS=%s|RECORDS=3\n",
           witness);
    fflush(stdout);
    return 0;
}
"#;
    const ENTITLEMENTS: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict><key>com.apple.security.app-sandbox</key><true/></dict></plist>
"#;
    pub struct Lab {
        dir: PathBuf,
        name: String,
        control_name: String,
    }
    impl Lab {
        pub fn new() -> Self {
            let n = SERIAL.fetch_add(1, Ordering::Relaxed);
            let d = env::temp_dir().join(format!("nordoi-r4l-{}-{n}", std::process::id()));
            fs::create_dir(&d).expect("fresh disposable test directory");
            Self {
                dir: fs::canonicalize(d).expect("canonical test fixture"),
                name: format!(
                    "org.eddarsglobal.nordoi.r4l.svc.p{}.n{n}",
                    std::process::id()
                ),
                control_name: format!(
                    "org.eddarsglobal.nordoi.r4l.open.p{}.n{n}",
                    std::process::id()
                ),
            }
        }
        fn app(&self) -> PathBuf {
            self.dir.join("R4LClient.app")
        }
        fn client(&self) -> PathBuf {
            self.app().join("Contents/MacOS/r4l-client")
        }
        fn good(&self) -> PathBuf {
            self.app().join("Contents/MacOS/r4l-good")
        }
        fn bad(&self) -> PathBuf {
            self.app().join("Contents/MacOS/r4l-bad")
        }
        fn service(&self) -> PathBuf {
            self.app()
                .join(format!("Contents/XPCServices/{}.xpc", self.name))
        }
        fn service_executable(&self) -> PathBuf {
            self.service().join("Contents/MacOS/r4l-service")
        }
        fn control_service(&self) -> PathBuf {
            self.app()
                .join(format!("Contents/XPCServices/{}.xpc", self.control_name))
        }
        fn control_executable(&self) -> PathBuf {
            self.control_service().join("Contents/MacOS/r4l-service")
        }
        fn journal(&self) -> PathBuf {
            self.dir.join("r4l-handler-receipts.txt")
        }
        fn run_command(path: &str, args: &[&str]) -> std::process::Output {
            Command::new(path).args(args).output().expect("macOS tool")
        }
        fn compile(&self, label: &str, src: &str, output: &Path) {
            let input = self.dir.join(label);
            fs::write(&input, src).expect("write test-only C source");
            let result = Command::new("/usr/bin/cc")
                .args(["-O2", "-std=c11", "-fblocks", "-x", "c"])
                .arg(&input)
                .arg("-o")
                .arg(output)
                .args(["-framework", "Foundation"])
                .output()
                .expect("Apple C compiler");
            assert!(
                result.status.success(),
                "native C build: {}",
                String::from_utf8_lossy(&result.stderr)
            );
        }
        fn sign(path: &Path, identifier: Option<&str>, ents: Option<&Path>) {
            let mut cmd = Command::new("/usr/bin/codesign");
            cmd.args(["--force", "--sign", "-"]);
            if let Some(id) = identifier {
                cmd.arg("--identifier").arg(id);
            }
            if let Some(ent) = ents {
                cmd.arg("--entitlements").arg(ent);
            }
            let r = cmd.arg(path).output().expect("codesign");
            assert!(
                r.status.success(),
                "ad-hoc signing: {}",
                String::from_utf8_lossy(&r.stderr)
            );
        }
        fn verify(path: &Path) {
            let r = Command::new("/usr/bin/codesign")
                .args(["--verify", "--strict"])
                .arg(path)
                .output()
                .expect("codesign verify");
            assert!(
                r.status.success(),
                "signature integrity: {}",
                String::from_utf8_lossy(&r.stderr)
            );
        }
        fn identity(path: &Path) -> String {
            let r = Command::new("/usr/bin/codesign")
                .args(["--display", "--verbose=2"])
                .arg(path)
                .output()
                .expect("codesign display");
            assert!(r.status.success(), "identity inspect failed");
            String::from_utf8_lossy(&r.stderr).into_owned()
        }
        pub fn prepare(&self) {
            let sentinel = self.dir.join("sentinel.txt");
            let mut f = OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(&sentinel)
                .unwrap();
            f.write_all(b"R4L_OWNER_ONLY\n").unwrap();
            OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(self.journal())
                .expect("fresh empty handler journal");
            fs::create_dir_all(self.app().join("Contents/MacOS")).unwrap();
            fs::create_dir_all(self.service().join("Contents/MacOS")).unwrap();
            fs::create_dir_all(self.control_service().join("Contents/MacOS")).unwrap();
            let parent = r#"<?xml version="1.0"?><plist version="1.0"><dict>
<key>CFBundleIdentifier</key><string>org.eddarsglobal.nordoi.r4l.client</string>
<key>CFBundleExecutable</key><string>r4l-client</string>
<key>CFBundlePackageType</key><string>APPL</string>
<key>CFBundleVersion</key><string>1</string>
<key>LSBackgroundOnly</key><true/>
</dict></plist>"#;
            let svc = format!(
                r#"<?xml version="1.0"?><plist version="1.0"><dict>
<key>CFBundleIdentifier</key><string>{}</string>
<key>CFBundleExecutable</key><string>r4l-service</string>
<key>CFBundlePackageType</key><string>XPC!</string>
<key>CFBundleVersion</key><string>1</string>
<key>XPCService</key><dict><key>RunLoopType</key><string>dispatch_main</string></dict>
</dict></plist>"#,
                self.name
            );
            fs::write(self.app().join("Contents/Info.plist"), parent).unwrap();
            fs::write(self.service().join("Contents/Info.plist"), &svc).unwrap();
            fs::write(
                self.control_service().join("Contents/Info.plist"),
                svc.replace(&self.name, &self.control_name),
            )
            .unwrap();
            let ent = self.dir.join("entitlements.plist");
            fs::write(&ent, ENTITLEMENTS).unwrap();
            self.compile("r4l_service.c", SERVICE, &self.service_executable());
            self.compile(
                "r4l_open_service.c",
                &format!("#define R4L_OPEN_CONTROL 1\n{SERVICE}"),
                &self.control_executable(),
            );
            self.compile("r4l_driver.c", DRIVER, &self.client());
            self.compile("r4l_peer.c", PEER, &self.good());
            fs::copy(self.good(), self.bad())
                .expect("copy IDENTICAL C source executable for adversary");
            // A normal signed sibling and an impostor from the SAME bundle,
            // directory, source bytes and account, with distinct signing IDs.
            Self::sign(
                &self.good(),
                Some("org.eddarsglobal.nordoi.r4l.client"),
                None,
            );
            Self::sign(
                &self.bad(),
                Some("org.eddarsglobal.nordoi.r4l.impostor"),
                None,
            );
            Self::sign(&self.service(), None, Some(&ent));
            Self::sign(&self.control_service(), None, Some(&ent));
            Self::sign(&self.app(), None, None);
            self.assert_fixture();
        }
        pub fn assert_fixture(&self) {
            for p in [
                self.app(),
                self.client(),
                self.good(),
                self.bad(),
                self.service(),
                self.service_executable(),
                self.control_service(),
                self.control_executable(),
            ] {
                Self::verify(&p);
            }
            let main = Self::identity(&self.app());
            let g = Self::identity(&self.good());
            let b = Self::identity(&self.bad());
            assert!(
                main.contains("Identifier=org.eddarsglobal.nordoi.r4l.client"),
                "main app signature identity must match admitted peer"
            );
            assert!(
                g.contains("Identifier=org.eddarsglobal.nordoi.r4l.client"),
                "valid peer identity"
            );
            assert!(
                b.contains("Identifier=org.eddarsglobal.nordoi.r4l.impostor"),
                "wrong peer identity"
            );
            assert_ne!(g, b, "two distinct independently signed identities");
            let s = Self::run_command(
                "/usr/bin/codesign",
                &[
                    "--display",
                    "--entitlements",
                    ":-",
                    self.service().to_str().unwrap(),
                ],
            );
            assert!(s.status.success(), "XPC service signed entitlements");
            let keys = String::from_utf8_lossy(&s.stdout);
            assert!(
                keys.contains("com.apple.security.app-sandbox"),
                "real sandbox"
            );
            assert_eq!(
                keys.matches("<key>").count(),
                1,
                "no sandbox privilege extras"
            );
            let control = Self::run_command(
                "/usr/bin/codesign",
                &[
                    "--display",
                    "--entitlements",
                    ":-",
                    self.control_service().to_str().unwrap(),
                ],
            );
            assert!(control.status.success(), "signed open-control entitlements");
            let control_keys = String::from_utf8_lossy(&control.stdout);
            assert_eq!(
                control_keys, keys,
                "open control and protected service must have identical entitlements"
            );
            assert_eq!(
                fs::read(self.dir.join("sentinel.txt")).unwrap(),
                b"R4L_OWNER_ONLY\n"
            );
        }
        pub fn execute(&self) -> Witness {
            let mut child = Command::new(self.client())
                .args([
                    self.name.as_str(),
                    self.control_name.as_str(),
                    self.good().to_str().unwrap(),
                    self.bad().to_str().unwrap(),
                    self.journal().to_str().unwrap(),
                ])
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .expect("run signed test app");
            let (ok, timed_out) = reap(&mut child);
            let mut out = String::new();
            let mut err = String::new();
            child
                .stdout
                .take()
                .unwrap()
                .take(LIMIT + 1)
                .read_to_string(&mut out)
                .unwrap();
            child
                .stderr
                .take()
                .unwrap()
                .take(LIMIT + 1)
                .read_to_string(&mut err)
                .unwrap();
            Witness {
                ok,
                timed_out,
                stdout: out,
                stderr: err,
            }
        }
    }
    impl Drop for Lab {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.dir).expect("remove test fixture only");
        }
    }
    pub struct Witness {
        pub ok: bool,
        pub timed_out: bool,
        pub stdout: String,
        pub stderr: String,
    }
    fn reap(child: &mut Child) -> (bool, bool) {
        let start = Instant::now();
        loop {
            if let Some(status) = child.try_wait().expect("child poll") {
                return (status.success(), false);
            }
            if start.elapsed() > Duration::from_secs(15) {
                let _ = child.kill();
                child.wait().expect("child reap");
                return (false, true);
            }
            thread::sleep(Duration::from_millis(10));
        }
    }
    pub fn assert_evidence(w: &Witness) {
        assert!(
            !w.timed_out && w.ok,
            "independent signed peer differential witness must succeed: {}",
            w.stderr
        );
        assert!(w.stdout.len() as u64 <= LIMIT, "bounded stdout");
        let lines: Vec<&str> = w.stdout.lines().collect();
        assert_eq!(lines.len(), 2, "two exact differential receipt frames");
        assert_eq!(
            lines[0],
            "R4L|GUARDED_GOOD=HANDLER_RECORDED|OPEN_BAD=HANDLER_RECORDED|POST_GOOD=HANDLER_RECORDED"
        );
        assert!(
            lines[1]=="R4L|GUARDED_BAD=NO_HANDLER_RECORD|WITNESS=SILENT_DROP|RECORDS=3"
             || lines[1]=="R4L|GUARDED_BAD=NO_HANDLER_RECORD|WITNESS=XPC_SIGNING_ERROR|RECORDS=3"
             || lines[1]=="R4L|GUARDED_BAD=NO_HANDLER_RECORD|WITNESS=CONNECTION_INVALID_DIFFERENTIAL|RECORDS=3"
             || lines[1]=="R4L|GUARDED_BAD=NO_HANDLER_RECORD|WITNESS=CONNECTION_INTERRUPTED_DIFFERENTIAL|RECORDS=3",
            "negative evidence must be paired with genuine protected and open-control XPC handler records"
        );
    }
    pub fn exercise() {
        let lab = Lab::new();
        lab.prepare();
        let w = lab.execute();
        assert_evidence(&w);
        let bytes = fs::read_to_string(lab.journal()).expect("read test-only handler receipts");
        let records: Vec<&str> = bytes.lines().collect();
        assert_eq!(
            records.len(),
            3,
            "three exact service-origin journal records"
        );
        for record in records {
            let pid = record
                .strip_prefix("R4L|HANDLER|PID=")
                .and_then(|v| v.parse::<u32>().ok());
            assert!(
                pid.is_some_and(|v| v != 0),
                "actual handler PID marker required"
            );
        }
        lab.assert_fixture();
    }
}

#[cfg(target_os = "macos")]
#[test]
fn native_two_independently_signed_peers_same_bundle_positive_and_negative() {
    native::exercise();
}
#[cfg(target_os = "macos")]
#[test]
fn native_repeated_independently_signed_peer_requirement_evidence() {
    for _ in 0..2 {
        native::exercise();
    }
}
#[cfg(target_os = "macos")]
#[test]
fn native_identity_signatures_and_sandbox_entitlements_are_distinct() {
    let lab = native::Lab::new();
    lab.prepare();
    lab.assert_fixture();
}
#[cfg(target_os = "macos")]
#[test]
fn forged_and_truncated_inbound_peer_reports_fail_closed() {
    use std::panic::{catch_unwind, AssertUnwindSafe};
    for v in [
        "",
        "R4L|GUARDED_GOOD=HANDLER_RECORDED|OPEN_BAD=HANDLER_RECORDED\n",
        "R4L|GUARDED_GOOD=HANDLER_RECORDED|OPEN_BAD=HANDLER_RECORDED\nR4L|GUARDED_BAD=HANDLER_RECORDED|WITNESS=SILENT_DROP|RECORDS=3\n",
        "R4L|GUARDED_GOOD=HANDLER_RECORDED|OPEN_BAD=HANDLER_RECORDED\nR4L|GUARDED_BAD=NO_HANDLER_RECORD|WITNESS=GENERIC_ERROR|RECORDS=2\n",
    ] {
        let w=native::Witness{ok:true,timed_out:false,stdout:v.to_string(),stderr:String::new()};
        assert!(catch_unwind(AssertUnwindSafe(||native::assert_evidence(&w))).is_err());
    }
}
#[cfg(not(target_os = "macos"))]
#[test]
fn non_macos_never_claims_native_xpc_peer_code_signing() {
    assert_ne!(std::env::consts::OS, "macos");
}
#[test]
fn reference_model_external_io_still_denied() {
    let (mut model, permit) = Model::bootstrap(
        0x4c,
        ScopeBudget {
            tasks: 0,
            resources: 0,
            children: 0,
        },
        16,
    );
    let root = model.root();
    assert_eq!(
        model.host_grant(&permit, root, ResourceEffect::ExternalIo),
        Err(ModelError::UnsupportedEffect)
    );
    assert_eq!(model.event_count(), 0);
}
