//! G0.3 R4K — research-only message-bound peer code signing identity and deny-by-default XPC policy.
//!
//! Test-only, disposable ad-hoc signed .app client and independently sandboxed
//! .xpc service. NO process-spawn fake or socket fallback; fails on unavailable
//! XPC registration. Not a production security mechanism or trusted .noi guest.
#![allow(dead_code)]

#[path = "../src/resource_task_r02.rs"]
mod reference;

use reference::{Model, ModelError, ResourceEffect, ScopeBudget};

#[cfg(target_os = "macos")]
mod macos {
    use std::env;
    use std::fs::{self, OpenOptions};
    use std::io::{Read, Write};
    use std::path::{Path, PathBuf};
    use std::process::{Child, Command, Stdio};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::thread;
    use std::time::{Duration, Instant};

    const CAP: u64 = 4096;
    static SERIAL: AtomicU64 = AtomicU64::new(1);

    const SERVICE_C: &str = r#"
#include <xpc/xpc.h>
#include <dispatch/dispatch.h>
#include <unistd.h>
#include <stdio.h>
#include <fcntl.h>
#include <string.h>
#include <stdlib.h>
#include <Security/Security.h>

static int authenticated_peer(xpc_object_t msg, OSStatus *code_status) {
    SecCodeRef code = NULL;
    SecRequirementRef req = NULL;
    OSStatus s = SecCodeCreateWithXPCMessage(msg, kSecCSDefaultFlags, &code);
    if (s != errSecSuccess || !code) { *code_status = (OSStatus)(1000000 + s); return 0; }
    s = SecRequirementCreateWithString(CFSTR("identifier \"org.eddarsglobal.nordoi.r4k.client\""), kSecCSDefaultFlags, &req);
    if (s != errSecSuccess || !req) {
        if (req) CFRelease(req);
        CFRelease(code);
        *code_status = (OSStatus)(3000000 + s);
        return 0;
    }
    s = SecCodeCheckValidity(code, kSecCSDefaultFlags, req);
    CFRelease(req);
    CFRelease(code);
    *code_status = (OSStatus)(4000000 + s);
    return s == errSecSuccess;
}

static void probe(const char *f, const char *c, int *rd, int *wr, int *cr) {
    *rd = *wr = *cr = 0;
    if (!f || !c) return;
    int fd = open(f, O_RDONLY);
    if (fd >= 0) { char b; *rd = (read(fd, &b, 1) == 1); close(fd); }
    fd = open(f, O_WRONLY);
    if (fd >= 0) { *wr = 1; close(fd); }
    fd = open(c, O_CREAT | O_EXCL | O_WRONLY, 0600);
    if (fd >= 0) { *cr = 1; close(fd); unlink(c); }
}
static void accept_peer(xpc_connection_t peer) {
    xpc_connection_set_event_handler(peer, ^(xpc_object_t msg) {
        if (xpc_get_type(msg) != XPC_TYPE_DICTIONARY) return;
        xpc_object_t reply = xpc_dictionary_create_reply(msg);
        if (!reply) return;
        const char *op = xpc_dictionary_get_string(msg, "op");
        OSStatus auth_error = 0;
        if (!authenticated_peer(msg, &auth_error)) {
            xpc_dictionary_set_string(reply, "status", "unauthenticated");
            xpc_dictionary_set_int64(reply, "auth_error", (int64_t)auth_error);
        } else if (!op || strcmp(op, "probe") != 0 ||
                   xpc_dictionary_get_value(msg, "grant") != NULL ||
                   xpc_dictionary_get_value(msg, "authority") != NULL ||
                   xpc_dictionary_get_value(msg, "identity") != NULL) {
            xpc_dictionary_set_string(reply, "status", "denied");
        } else {
            int rd=0, wr=0, cr=0;
            probe(xpc_dictionary_get_string(msg, "sentinel"),
                  xpc_dictionary_get_string(msg, "create"), &rd, &wr, &cr);
            xpc_dictionary_set_string(reply, "status", "probed");
            xpc_dictionary_set_int64(reply, "pid", (int64_t)getpid());
            xpc_dictionary_set_int64(reply, "read", rd);
            xpc_dictionary_set_int64(reply, "write", wr);
            xpc_dictionary_set_int64(reply, "create", cr);
        }
        xpc_connection_send_message(peer, reply);
        xpc_release(reply);
    });
    xpc_connection_resume(peer);
}
int main(void) {
    xpc_main(accept_peer);
    return 0;
}
"#;

    const CLIENT_C: &str = r#"
#include <xpc/xpc.h>
#include <dispatch/dispatch.h>
#include <unistd.h>
#include <stdio.h>
#include <fcntl.h>
#include <stdlib.h>
#include <string.h>
#include <inttypes.h>
static void probe(const char *f, const char *c, int *rd, int *wr, int *cr) {
    *rd=*wr=*cr=0;
    int fd=open(f, O_RDONLY);
    if (fd>=0) { char b; *rd=(read(fd,&b,1)==1); close(fd); }
    fd=open(f, O_WRONLY);
    if (fd>=0) { *wr=1; close(fd); }
    fd=open(c, O_CREAT|O_EXCL|O_WRONLY,0600);
    if (fd>=0) { *cr=1; close(fd); unlink(c); }
}
int main(int argc, char **argv) {
    if(argc!=4) return 80;
    int rd=0,wr=0,cr=0;
    probe(argv[2],argv[3],&rd,&wr,&cr);
    printf("R4K|CLIENT|PID=%ld|READ=%d|WRITE=%d|CREATE=%d\n",
           (long)getpid(), rd, wr, cr);
    fflush(stdout);
    xpc_connection_t con=xpc_connection_create(argv[1],NULL);
    if(!con) return 81;
    xpc_connection_set_event_handler(con, ^(xpc_object_t e) { (void)e; });
    xpc_connection_resume(con);
    xpc_object_t q=xpc_dictionary_create(NULL,NULL,0);
    xpc_dictionary_set_string(q,"op","probe");
    xpc_dictionary_set_string(q,"sentinel",argv[2]);
    xpc_dictionary_set_string(q,"create",argv[3]);
    xpc_object_t result=xpc_connection_send_message_with_reply_sync(con,q);
    xpc_release(q);
    if(!result || xpc_get_type(result)!=XPC_TYPE_DICTIONARY) {
        fprintf(stderr,"R4K|XPC_UNAVAILABLE\n");
        if(result) xpc_release(result);
        xpc_connection_cancel(con);
        xpc_release(con);
        return 82;
    }
    const char *status=xpc_dictionary_get_string(result,"status");
    int64_t pid=xpc_dictionary_get_int64(result,"pid");
    int64_t r=xpc_dictionary_get_int64(result,"read");
    int64_t w=xpc_dictionary_get_int64(result,"write");
    int64_t c=xpc_dictionary_get_int64(result,"create");
    if(!status || strcmp(status,"probed") || pid<=0 || pid==getpid()) {
        fprintf(stderr, "R4K|AUTH_DIAGNOSTIC|STATUS=%s|OSSTATUS=%" PRId64 "\\n",
                status?status:"NULL", xpc_dictionary_get_int64(result,"auth_error"));
        xpc_release(result);
        xpc_connection_cancel(con);
        xpc_release(con);
        return 83;
    }
    printf("R4K|SERVICE|PID=%" PRId64 "|READ=%" PRId64 "|WRITE=%" PRId64 "|CREATE=%" PRId64 "\n",
           pid,r,w,c);
    xpc_release(result);

    // A malicious operation name cannot request a filesystem write.
    xpc_object_t invalid=xpc_dictionary_create(NULL,NULL,0);
    xpc_dictionary_set_string(invalid,"op","writefile");
    xpc_dictionary_set_string(invalid,"sentinel",argv[2]);
    xpc_dictionary_set_string(invalid,"create",argv[3]);
    xpc_object_t denied=xpc_connection_send_message_with_reply_sync(con,invalid);
    xpc_release(invalid);
    if(!denied || xpc_get_type(denied)!=XPC_TYPE_DICTIONARY) return 84;
    const char *verdict=xpc_dictionary_get_string(denied,"status");
    if(!verdict || strcmp(verdict,"denied")) return 85;
    printf("R4K|POLICY|UNKNOWN=DENIED\n");
    // Reject fake client-granted authority even when sent over authenticated XPC.
    xpc_object_t forged=xpc_dictionary_create(NULL,NULL,0);
    xpc_dictionary_set_string(forged,"op","probe");
    xpc_dictionary_set_string(forged,"sentinel",argv[2]);
    xpc_dictionary_set_string(forged,"create",argv[3]);
    xpc_dictionary_set_string(forged,"grant","unlimited");
    xpc_dictionary_set_string(forged,"identity","trusted");
    xpc_object_t fr=xpc_connection_send_message_with_reply_sync(con,forged);
    xpc_release(forged);
    if(!fr || xpc_get_type(fr)!=XPC_TYPE_DICTIONARY) return 86;
    const char *fs=xpc_dictionary_get_string(fr,"status");
    if(!fs || strcmp(fs,"denied")) return 87;
    xpc_release(fr);
    printf("R4K|POLICY|FORGED_AUTHORITY=DENIED\\n");

    fflush(stdout);
    xpc_release(denied);
    xpc_connection_cancel(con);
    xpc_release(con);
    return 0;
}
"#;
    const SANDBOX_PLIST: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict><key>com.apple.security.app-sandbox</key><true/></dict></plist>
"#;

    pub struct Lab {
        pub dir: PathBuf,
        service_name: String,
    }
    impl Lab {
        pub fn new() -> Self {
            let n = SERIAL.fetch_add(1, Ordering::Relaxed);
            let pid = std::process::id();
            let directory = env::temp_dir().join(format!("nordoi-r4k-{pid}-{n}"));
            fs::create_dir(&directory).expect("new disposable owner-owned fixture");
            Self {
                dir: fs::canonicalize(directory).expect("canonicalize temp fixture"),
                service_name: format!("org.eddarsglobal.nordoi.r4k.service.p{pid}.n{n}"),
            }
        }
        pub fn app(&self) -> PathBuf {
            self.dir.join("R4KClient.app")
        }
        pub fn client(&self) -> PathBuf {
            self.app().join("Contents/MacOS/r4k-client")
        }
        pub fn service_bundle(&self) -> PathBuf {
            self.app()
                .join(format!("Contents/XPCServices/{}.xpc", self.service_name))
        }
        pub fn service_exec(&self) -> PathBuf {
            self.service_bundle().join("Contents/MacOS/r4k-service")
        }
        pub fn sentinel(&self) -> PathBuf {
            self.dir.join("parent-owned.txt")
        }
        pub fn creation(&self) -> PathBuf {
            self.dir.join("child-must-not-create.txt")
        }
        fn path(&self, name: &str) -> PathBuf {
            self.dir.join(name)
        }

        fn compile(&self, name: &str, content: &str, binary: &Path) {
            let path = self.path(name);
            fs::write(&path, content).expect("write fixed native test-only source");
            let build = Command::new("/usr/bin/cc")
                .args(["-O2", "-std=c11", "-fblocks", "-x", "c"])
                .arg(path)
                .arg("-o")
                .arg(binary)
                .args(["-framework", "Foundation", "-framework", "Security"])
                .output()
                .expect("Apple command-line tools required");
            assert!(
                build.status.success(),
                "compile real libxpc test-only helper: {}",
                String::from_utf8_lossy(&build.stderr)
            );
        }
        fn sign(path: &Path, ent: Option<&Path>) {
            let mut command = Command::new("/usr/bin/codesign");
            command.args(["--force", "--sign", "-"]);
            if let Some(ent) = ent {
                command.arg("--entitlements").arg(ent);
            }
            let result = command.arg(path).output().expect("codesign required");
            assert!(
                result.status.success(),
                "native ad-hoc test bundle signing: {}",
                String::from_utf8_lossy(&result.stderr)
            );
        }
        fn verify_signature(path: &Path) {
            let result = Command::new("/usr/bin/codesign")
                .args(["--verify", "--strict", "--verbose=2"])
                .arg(path)
                .output()
                .expect("codesign verify");
            assert!(
                result.status.success(),
                "invalid test code signature: {}",
                String::from_utf8_lossy(&result.stderr)
            );
        }
        fn signed_keys(path: &Path) -> String {
            let output = Command::new("/usr/bin/codesign")
                .args(["--display", "--entitlements", ":-"])
                .arg(path)
                .output()
                .expect("codesign signed entitlements");
            assert!(output.status.success(), "entitlements readback unavailable");
            String::from_utf8_lossy(&output.stdout).into_owned()
        }
        pub fn prepare(&self) {
            let mut f = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(self.sentinel())
                .expect("fresh sentinel");
            f.write_all(b"R4K_PARENT_ONLY\n").unwrap();
            let contents = self.app().join("Contents");
            fs::create_dir_all(contents.join("MacOS")).unwrap();
            let svc = self.service_bundle().join("Contents");
            fs::create_dir_all(svc.join("MacOS")).unwrap();

            let client_info = r#"<?xml version="1.0" encoding="UTF-8"?>
<plist version="1.0"><dict>
<key>CFBundleIdentifier</key><string>org.eddarsglobal.nordoi.r4k.client</string>
<key>CFBundleExecutable</key><string>r4k-client</string>
<key>CFBundlePackageType</key><string>APPL</string>
<key>CFBundleVersion</key><string>1</string>
<key>LSBackgroundOnly</key><true/>
</dict></plist>"#;
            let svc_info=format!(
                "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
                <plist version=\"1.0\"><dict>\
                <key>CFBundleIdentifier</key><string>{}</string>\
                <key>CFBundleExecutable</key><string>r4k-service</string>\
                <key>CFBundlePackageType</key><string>XPC!</string>\
                <key>CFBundleVersion</key><string>1</string>\
                <key>XPCService</key><dict><key>RunLoopType</key><string>dispatch_main</string></dict>\
                </dict></plist>",
                self.service_name
            );
            fs::write(contents.join("Info.plist"), client_info).unwrap();
            fs::write(svc.join("Info.plist"), svc_info).unwrap();
            fs::write(self.path("service.entitlements"), SANDBOX_PLIST).unwrap();
            self.compile("xpc_service.c", SERVICE_C, &self.service_exec());
            self.compile("xpc_client.c", CLIENT_C, &self.client());
            // Nested signed service before outer signed app, no extra entitlement.
            Self::sign(
                &self.service_bundle(),
                Some(&self.path("service.entitlements")),
            );
            Self::sign(&self.app(), None);
            self.verify();
            self.check_fixture();
        }
        pub fn verify(&self) {
            Self::verify_signature(&self.app());
            Self::verify_signature(&self.client());
            Self::verify_signature(&self.service_bundle());
            Self::verify_signature(&self.service_exec());
            let service = Self::signed_keys(&self.service_bundle());
            let client = Self::signed_keys(&self.app());
            assert!(
                service.contains("<key>com.apple.security.app-sandbox</key>"),
                "XPC service must be signed with OS App Sandbox"
            );
            assert_eq!(
                service.matches("<key>").count(),
                1,
                "service must have no extra privilege entitlements"
            );
            assert!(
                !client.contains("com.apple.security.app-sandbox"),
                "client control must not itself be sandboxed"
            );
        }
        pub fn check_fixture(&self) {
            assert_eq!(fs::read(self.sentinel()).unwrap(), b"R4K_PARENT_ONLY\n");
            assert!(!self.creation().exists());
        }
    }
    impl Drop for Lab {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.dir).expect("remove only our fresh test fixture");
        }
    }
    pub struct Witness {
        pub ok: bool,
        pub timed_out: bool,
        pub stdout: String,
        pub stderr: String,
    }
    fn wait_reap(child: &mut Child) -> (bool, bool) {
        let start = Instant::now();
        loop {
            if let Some(exit) = child.try_wait().expect("XPC client status") {
                return (exit.success(), false);
            }
            if start.elapsed() > Duration::from_secs(15) {
                let _ = child.kill();
                child.wait().expect("XPC client must be reaped");
                return (false, true);
            }
            thread::sleep(Duration::from_millis(10));
        }
    }
    pub fn run(lab: &Lab) -> Witness {
        let mut child = Command::new(lab.client())
            .arg(&lab.service_name)
            .arg(lab.sentinel())
            .arg(lab.creation())
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("run only signed test client within disposable .app");
        let (ok, timed_out) = wait_reap(&mut child);
        let mut stdout = String::new();
        let mut stderr = String::new();
        child
            .stdout
            .take()
            .expect("child stdout")
            .take(CAP + 1)
            .read_to_string(&mut stdout)
            .expect("UTF-8 stdout");
        child
            .stderr
            .take()
            .expect("child stderr")
            .take(CAP + 1)
            .read_to_string(&mut stderr)
            .expect("UTF-8 stderr");
        Witness {
            ok,
            timed_out,
            stdout,
            stderr,
        }
    }
    fn pid(line: &str, prefix: &str) -> Option<u32> {
        let rest = line.strip_prefix(prefix)?;
        let token = rest.split('|').next()?;
        token.parse::<u32>().ok().filter(|p| *p != 0)
    }
    pub fn assert_xpc(w: &Witness) {
        assert!(
            w.ok && !w.timed_out,
            "XPC client and service MUST complete via real launchd: {}",
            w.stderr
        );
        assert!(w.stdout.len() as u64 <= CAP, "bounded receipt");
        let lines: Vec<&str> = w.stdout.lines().collect();
        assert_eq!(
            lines.len(),
            4,
            "exactly 4 XPC receipt frames required: {}",
            w.stdout
        );
        let cp = pid(lines[0], "R4K|CLIENT|PID=").expect("real client pid");
        let sp = pid(lines[1], "R4K|SERVICE|PID=").expect("real service pid");
        assert_ne!(cp, sp, "XPC server MUST be distinct OS process");
        assert!(
            lines[0].ends_with("|READ=1|WRITE=1|CREATE=1"),
            "parent baseline must actually have authority"
        );
        assert!(
            lines[1].ends_with("|READ=0|WRITE=0|CREATE=0"),
            "XPC service MUST be kernel-denied even for known path"
        );
        assert_eq!(
            lines[2], "R4K|POLICY|UNKNOWN=DENIED",
            "unrecognized/host write operation must fail closed"
        );
        assert_eq!(lines[3], "R4K|POLICY|FORGED_AUTHORITY=DENIED");
    }
    pub fn exercise() {
        let lab = Lab::new();
        lab.prepare();
        let response = run(&lab);
        assert_xpc(&response);
        lab.check_fixture();
    }
}

#[cfg(target_os = "macos")]
#[test]
fn macos_real_bundled_xpc_service_runs_separately_and_denies_host_io() {
    macos::exercise();
}
#[cfg(target_os = "macos")]
#[test]
fn macos_repeated_xpc_activation_does_not_gain_client_file_authority() {
    for _ in 0..2 {
        macos::exercise();
    }
}
#[cfg(target_os = "macos")]
#[test]
fn macos_xpc_service_and_client_have_separate_signed_entitlement_sets() {
    let lab = macos::Lab::new();
    lab.prepare();
    lab.verify();
}
#[cfg(target_os = "macos")]
#[test]
fn macos_fail_closed_forged_ipc_receipts_and_duplicate_server_frame() {
    use std::panic::{catch_unwind, AssertUnwindSafe};
    for frame in [
        "",
        "R4K|CLIENT|PID=1|READ=1|WRITE=1|CREATE=1\nR4K|SERVICE|PID=1|READ=0|WRITE=0|CREATE=0\nR4K|POLICY|UNKNOWN=DENIED\nR4K|POLICY|FORGED_AUTHORITY=DENIED\n",
        "R4K|CLIENT|PID=1|READ=1|WRITE=1|CREATE=1\nR4K|SERVICE|PID=2|READ=1|WRITE=0|CREATE=0\nR4K|POLICY|UNKNOWN=DENIED\nR4K|POLICY|FORGED_AUTHORITY=DENIED\n",
        "R4K|CLIENT|PID=1|READ=1|WRITE=1|CREATE=1\nR4K|SERVICE|PID=2|READ=0|WRITE=0|CREATE=0\nR4K|POLICY|UNKNOWN=ALLOWED\n",
    ] {
        let receipt=macos::Witness {ok:true,timed_out:false,stdout:frame.into(),stderr:String::new()};
        assert!(catch_unwind(AssertUnwindSafe(|| macos::assert_xpc(&receipt))).is_err());
    }
}
#[cfg(target_os = "macos")]
#[test]
fn macos_unsandboxed_test_driver_keeps_fresh_sentinel_ownership() {
    use std::fs::OpenOptions;
    use std::io::Write;
    let lab = macos::Lab::new();
    lab.prepare();
    macos::assert_xpc(&macos::run(&lab));
    let mut f = OpenOptions::new()
        .append(true)
        .open(lab.sentinel())
        .unwrap();
    f.write_all(b"DRIVER_CAN_APPEND\n").unwrap();
    assert_eq!(
        std::fs::read(lab.sentinel()).unwrap(),
        b"R4K_PARENT_ONLY\nDRIVER_CAN_APPEND\n"
    );
}
#[cfg(target_os = "macos")]
#[test]
fn macos_r4k_authentication_is_bound_to_actual_xpc_messages() {
    let lab = macos::Lab::new();
    lab.prepare();
    let receipt = macos::run(&lab);
    macos::assert_xpc(&receipt);
    assert!(receipt.stdout.contains("R4K|POLICY|FORGED_AUTHORITY=DENIED"));
    lab.check_fixture();
}
#[cfg(not(target_os = "macos"))]
#[test]
fn portable_model_never_claims_macos_xpc_sandbox() {
    assert_ne!(std::env::consts::OS, "macos");
}
#[test]
fn reference_external_io_is_still_unsupported() {
    let (mut model, permit) = Model::bootstrap(
        0x4a,
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
