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
    int status=xpc_connection_set_peer_code_signing_requirement(peer, CLIENT_REQ);
    if(status!=0) { xpc_connection_cancel(peer); return; }
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
                  xpc_dictionary_get_count(msg)==2 &&
                  xpc_dictionary_get_string(msg,"nonce") &&
                  strcmp(xpc_dictionary_get_string(msg,"nonce"),"r4l-fixed-probe")==0) {
            atomic_fetch_add_explicit(&accepted,1,memory_order_relaxed);
            xpc_dictionary_set_string(reply,"status","pong");
            xpc_dictionary_set_int64(reply,"pid",(int64_t)getpid());
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
int main(int argc,char **argv) {
    if(argc!=4) return 70;
    if(strcmp(argv[3],"good")!=0 && strcmp(argv[3],"bad")!=0) return 71;
    alarm(4);
    xpc_connection_t c=xpc_connection_create(argv[1],NULL);
    if(!c) return 72;
    xpc_connection_set_event_handler(c,^(xpc_object_t msg){ (void)msg; });
    xpc_connection_resume(c);
    xpc_object_t req=xpc_dictionary_create(NULL,NULL,0);
    xpc_dictionary_set_string(req,"op","ping");
    xpc_dictionary_set_string(req,"nonce",argv[2]);
    xpc_object_t resp=xpc_connection_send_message_with_reply_sync(c,req);
    xpc_release(req);
    int result=73;
    if(resp && xpc_get_type(resp)==XPC_TYPE_DICTIONARY) {
        const char *s=xpc_dictionary_get_string(resp,"status");
        if(s && strcmp(s,"pong")==0) {
            result=(strcmp(argv[3],"good")==0)?0:74;
        }
    } else if(resp==XPC_ERROR_PEER_CODE_SIGNING_REQUIREMENT) {
        result=(strcmp(argv[3],"bad")==0)?7:75;
    } else if(resp && xpc_get_type(resp)==XPC_TYPE_ERROR) {
        fprintf(stderr,"R4L|UNEXPECTED_XPC_ERROR|PEER=%s\n",argv[3]);
        result=(strcmp(argv[3],"bad")==0)?8:75;
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
#include <unistd.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
extern char **environ;
static int count(xpc_connection_t c, uint64_t *n, int64_t *pid) {
    xpc_object_t q=xpc_dictionary_create(NULL,NULL,0);
    xpc_dictionary_set_string(q,"op","count");
    xpc_object_t r=xpc_connection_send_message_with_reply_sync(c,q);
    xpc_release(q);
    if(!r || xpc_get_type(r)!=XPC_TYPE_DICTIONARY) {
        fprintf(stderr,"R4L|COUNTER_NOT_DICTIONARY|IS_NULL=%d|IS_ERROR=%d\n",
            r==NULL,r && xpc_get_type(r)==XPC_TYPE_ERROR);
        if(r) xpc_release(r);
        return 0;
    }
    const char *s=xpc_dictionary_get_string(r,"status");
    int valid=s && strcmp(s,"counted")==0;
    if(!valid) {
        fprintf(stderr,"R4L|COUNTER_BAD_STATUS|VALUE=%.60s|KEYS=%zu\n",
                s?s:"NULL",xpc_dictionary_get_count(r));
    }
    if(valid) {
        *n=xpc_dictionary_get_uint64(r,"accepted");
        *pid=xpc_dictionary_get_int64(r,"pid");
    }
    xpc_release(r);
    return valid;
}
static int run_peer(char *exe,char *svc,char *mode) {
    pid_t pid=0;
    char *const args[]={exe,svc,"r4l-fixed-probe",mode,NULL};
    if(posix_spawn(&pid,exe,NULL,NULL,args,environ)!=0) return -1;
    int status=0;
    if(waitpid(pid,&status,0)!=pid) return -2;
    if(WIFEXITED(status)) return WEXITSTATUS(status);
    fprintf(stderr,"R4L|PEER_SIGNAL_EXIT|SIGNAL=%d\n",
            WIFSIGNALED(status)?WTERMSIG(status):-1);
    return -3;
}
int main(int argc,char **argv) {
    if(argc!=4) return 80;
    xpc_connection_t c=xpc_connection_create(argv[1],NULL);
    if(!c) return 81;
    xpc_connection_set_event_handler(c,^(xpc_object_t m){(void)m;});
    xpc_connection_resume(c);
    uint64_t before=999,after_good=999,after_bad=999;
    int64_t p0=0,p1=0,p2=0;
    if(!count(c,&before,&p0) || before!=0 || p0<=0 || p0==getpid()) {
        fprintf(stderr,"R4L|BASELINE_COUNT_FAILED|COUNTER=%llu|PID=%lld\n",
                (unsigned long long)before,(long long)p0);
        return 82;
    }
    int good=run_peer(argv[2],argv[1],"good");
    if(good!=0) {fprintf(stderr,"R4L|GOOD_SIBLING_UNREACHABLE|EXIT=%d\n",good);return 83;}
    if(!count(c,&after_good,&p1) || after_good!=0 || p1!=p0) {
        fprintf(stderr,"R4L|COORDINATOR_INSTANCE_CHANGED|COUNTER=%llu|PID=%lld|BASEPID=%lld\n",
                (unsigned long long)after_good,(long long)p1,(long long)p0);
        return 84;
    }
    int bad=run_peer(argv[3],argv[1],"bad");
    if(bad!=7) {fprintf(stderr,"R4L|BAD_SIBLING_NOT_EXPLICIT_SIGNATURE_ERROR|EXIT=%d\n",bad);return 85;}
    if(!count(c,&after_bad,&p2) || after_bad!=0 || p2!=p0) {
        fprintf(stderr,"R4L|UNAUTHORIZED_MESSAGE_REACHED_SERVICE|COUNT=%llu\n",
                (unsigned long long)after_bad);
        return 86;
    }
    printf("R4L|CLIENT=%ld|SERVER=%lld\n",(long)getpid(),(long long)p0);
    printf("R4L|GOOD=ACCEPTED|BAD=EXPLICIT_SIGNING_ERROR|BEFORE=%llu|AFTER_GOOD=%llu|AFTER_BAD=%llu\n",
           (unsigned long long)before,(unsigned long long)after_good,
           (unsigned long long)after_bad);
    fflush(stdout);
    xpc_connection_cancel(c);
    xpc_release(c);
    return 0;
}
"#;
    const ENTITLEMENTS: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict><key>com.apple.security.app-sandbox</key><true/></dict></plist>
"#;
    pub struct Lab { dir: PathBuf, name: String }
    impl Lab {
        pub fn new() -> Self {
            let n=SERIAL.fetch_add(1,Ordering::Relaxed);
            let d=env::temp_dir().join(format!("nordoi-r4l-{}-{n}",std::process::id()));
            fs::create_dir(&d).expect("fresh disposable test directory");
            Self {
                dir:fs::canonicalize(d).expect("canonical test fixture"),
                name:format!("org.eddarsglobal.nordoi.r4l.svc.p{}.n{n}",std::process::id())
            }
        }
        fn app(&self)->PathBuf {self.dir.join("R4LClient.app")}
        fn client(&self)->PathBuf {self.app().join("Contents/MacOS/r4l-client")}
        fn good(&self)->PathBuf {self.app().join("Contents/MacOS/r4l-good")}
        fn bad(&self)->PathBuf {self.app().join("Contents/MacOS/r4l-bad")}
        fn service(&self)->PathBuf {self.app().join(format!("Contents/XPCServices/{}.xpc",self.name))}
        fn service_executable(&self)->PathBuf {self.service().join("Contents/MacOS/r4l-service")}
        fn run_command(path:&str,args:&[&str])->std::process::Output {
            Command::new(path).args(args).output().expect("macOS tool")
        }
        fn compile(&self,label:&str,src:&str,output:&Path) {
            let input=self.dir.join(label);
            fs::write(&input,src).expect("write test-only C source");
            let result=Command::new("/usr/bin/cc")
                .args(["-O2","-std=c11","-fblocks","-x","c"]).arg(&input)
                .arg("-o").arg(output)
                .args(["-framework","Foundation"])
                .output().expect("Apple C compiler");
            assert!(result.status.success(),"native C build: {}",String::from_utf8_lossy(&result.stderr));
        }
        fn sign(path:&Path,identifier:Option<&str>,ents:Option<&Path>) {
            let mut cmd=Command::new("/usr/bin/codesign");
            cmd.args(["--force","--sign","-"]);
            if let Some(id)=identifier {cmd.arg("--identifier").arg(id);}
            if let Some(ent)=ents {cmd.arg("--entitlements").arg(ent);}
            let r=cmd.arg(path).output().expect("codesign");
            assert!(r.status.success(),"ad-hoc signing: {}",String::from_utf8_lossy(&r.stderr));
        }
        fn verify(path:&Path) {
            let r=Command::new("/usr/bin/codesign")
                .args(["--verify","--strict"]).arg(path).output().expect("codesign verify");
            assert!(r.status.success(),"signature integrity: {}",String::from_utf8_lossy(&r.stderr));
        }
        fn identity(path:&Path)->String {
            let r=Command::new("/usr/bin/codesign")
                .args(["--display","--verbose=2"]).arg(path).output().expect("codesign display");
            assert!(r.status.success(),"identity inspect failed");
            String::from_utf8_lossy(&r.stderr).into_owned()
        }
        pub fn prepare(&self) {
            let sentinel=self.dir.join("sentinel.txt");
            let mut f=OpenOptions::new().create_new(true).write(true).open(&sentinel).unwrap();
            f.write_all(b"R4L_OWNER_ONLY\n").unwrap();
            fs::create_dir_all(self.app().join("Contents/MacOS")).unwrap();
            fs::create_dir_all(self.service().join("Contents/MacOS")).unwrap();
            let parent=r#"<?xml version="1.0"?><plist version="1.0"><dict>
<key>CFBundleIdentifier</key><string>org.eddarsglobal.nordoi.r4l.client</string>
<key>CFBundleExecutable</key><string>r4l-client</string>
<key>CFBundlePackageType</key><string>APPL</string>
<key>CFBundleVersion</key><string>1</string>
<key>LSBackgroundOnly</key><true/>
</dict></plist>"#;
            let svc=format!(r#"<?xml version="1.0"?><plist version="1.0"><dict>
<key>CFBundleIdentifier</key><string>{}</string>
<key>CFBundleExecutable</key><string>r4l-service</string>
<key>CFBundlePackageType</key><string>XPC!</string>
<key>CFBundleVersion</key><string>1</string>
<key>XPCService</key><dict><key>RunLoopType</key><string>dispatch_main</string></dict>
</dict></plist>"#,self.name);
            fs::write(self.app().join("Contents/Info.plist"),parent).unwrap();
            fs::write(self.service().join("Contents/Info.plist"),svc).unwrap();
            let ent=self.dir.join("entitlements.plist");
            fs::write(&ent,ENTITLEMENTS).unwrap();
            self.compile("r4l_service.c",SERVICE,&self.service_executable());
            self.compile("r4l_driver.c",DRIVER,&self.client());
            self.compile("r4l_peer.c",PEER,&self.good());
            fs::copy(self.good(),self.bad()).expect("copy IDENTICAL C source executable for adversary");
            // A normal signed sibling and an impostor from the SAME bundle,
            // directory, source bytes and account, with distinct signing IDs.
            Self::sign(&self.good(),Some("org.eddarsglobal.nordoi.r4l.client"),None);
            Self::sign(&self.bad(),Some("org.eddarsglobal.nordoi.r4l.impostor"),None);
            Self::sign(&self.service(),None,Some(&ent));
            Self::sign(&self.app(),None,None);
            self.assert_fixture();
        }
        pub fn assert_fixture(&self) {
            for p in [self.app(),self.client(),self.good(),self.bad(),
                      self.service(),self.service_executable()] {Self::verify(&p);}
            let main=Self::identity(&self.app());
            let g=Self::identity(&self.good());
            let b=Self::identity(&self.bad());
            assert!(
                main.contains("Identifier=org.eddarsglobal.nordoi.r4l.client"),
                "main app signature identity must match admitted peer"
            );
            assert!(g.contains("Identifier=org.eddarsglobal.nordoi.r4l.client"),"valid peer identity");
            assert!(b.contains("Identifier=org.eddarsglobal.nordoi.r4l.impostor"),"wrong peer identity");
            assert_ne!(g,b,"two distinct independently signed identities");
            let s=Self::run_command("/usr/bin/codesign",&[
                "--display","--entitlements",":-",self.service().to_str().unwrap()]);
            assert!(s.status.success(),"XPC service signed entitlements");
            let keys=String::from_utf8_lossy(&s.stdout);
            assert!(keys.contains("com.apple.security.app-sandbox"),"real sandbox");
            assert_eq!(keys.matches("<key>").count(),1,"no sandbox privilege extras");
            assert_eq!(fs::read(self.dir.join("sentinel.txt")).unwrap(),b"R4L_OWNER_ONLY\n");
        }
        pub fn execute(&self)->Witness {
            let mut child=Command::new(self.client())
                .args([self.name.as_str(),self.good().to_str().unwrap(),self.bad().to_str().unwrap()])
                .stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped())
                .spawn().expect("run signed test app");
            let (ok,timed_out)=reap(&mut child);
            let mut out=String::new();
            let mut err=String::new();
            child.stdout.take().unwrap().take(LIMIT+1).read_to_string(&mut out).unwrap();
            child.stderr.take().unwrap().take(LIMIT+1).read_to_string(&mut err).unwrap();
            Witness{ok,timed_out,stdout:out,stderr:err}
        }
    }
    impl Drop for Lab {
        fn drop(&mut self) {fs::remove_dir_all(&self.dir).expect("remove test fixture only");}
    }
    pub struct Witness {
        pub ok:bool,pub timed_out:bool,pub stdout:String,pub stderr:String
    }
    fn reap(child:&mut Child)->(bool,bool) {
        let start=Instant::now();
        loop {
            if let Some(status)=child.try_wait().expect("child poll") {
                return (status.success(),false);
            }
            if start.elapsed()>Duration::from_secs(15) {
                let _=child.kill();
                child.wait().expect("child reap");
                return (false,true);
            }
            thread::sleep(Duration::from_millis(10));
        }
    }
    pub fn assert_evidence(w:&Witness) {
        assert!(!w.timed_out && w.ok,"native signed peer run MUST succeed: {}",w.stderr);
        assert!(w.stdout.len() as u64<=LIMIT,"bounded stdout");
        let lines:Vec<&str>=w.stdout.lines().collect();
        assert_eq!(lines.len(),2,"exact two native receipt frames: {}",w.stdout);
        assert!(lines[0].starts_with("R4L|CLIENT=") && lines[0].contains("|SERVER="));
        let chunks:Vec<&str>=lines[0].split('|').collect();
        assert_eq!(chunks.len(),3);
        let cp=chunks[1].strip_prefix("CLIENT=").unwrap().parse::<u32>().unwrap();
        let sp=chunks[2].strip_prefix("SERVER=").unwrap().parse::<u32>().unwrap();
        assert_ne!(cp,sp,"service must be distinct OS process");
        assert_eq!(
            lines[1],
            "R4L|GOOD=ACCEPTED|BAD=EXPLICIT_SIGNING_ERROR|BEFORE=0|AFTER_GOOD=0|AFTER_BAD=0",
            "genuine good sibling must succeed and wrong-signed sibling must return the specific XPC signing-error constant"
        );
    }
    pub fn exercise() {
        let lab=Lab::new();
        lab.prepare();
        let w=lab.execute();
        assert_evidence(&w);
        lab.assert_fixture();
    }
}

#[cfg(target_os="macos")]
#[test]
fn native_two_independently_signed_peers_same_bundle_positive_and_negative() {
    native::exercise();
}
#[cfg(target_os="macos")]
#[test]
fn native_repeated_signed_peers_do_not_reach_unauthorized_handler() {
    for _ in 0..2 { native::exercise(); }
}
#[cfg(target_os="macos")]
#[test]
fn native_identity_signatures_and_sandbox_entitlements_are_distinct() {
    let lab=native::Lab::new();
    lab.prepare();
    lab.assert_fixture();
}
#[cfg(target_os="macos")]
#[test]
fn forged_and_truncated_inbound_peer_reports_fail_closed() {
    use std::panic::{catch_unwind,AssertUnwindSafe};
    for v in [
        "",
        "R4L|CLIENT=1|SERVER=1\nR4L|GOOD=ACCEPTED|BAD=XPC_ERROR|BEFORE=0|AFTER_GOOD=1|AFTER_BAD=1\n",
        "R4L|CLIENT=1|SERVER=2\nR4L|GOOD=ACCEPTED|BAD=ACCEPTED|BEFORE=0|AFTER_GOOD=0|AFTER_BAD=1\n",
        "R4L|CLIENT=1|SERVER=2\nR4L|GOOD=ACCEPTED|BAD=EXPLICIT_SIGNING_ERROR|BEFORE=0|AFTER_GOOD=0|AFTER_BAD=1\n",
    ] {
        let w=native::Witness{ok:true,timed_out:false,stdout:v.to_string(),stderr:String::new()};
        assert!(catch_unwind(AssertUnwindSafe(||native::assert_evidence(&w))).is_err());
    }
}
#[cfg(not(target_os="macos"))]
#[test]
fn non_macos_never_claims_native_xpc_peer_code_signing() {
    assert_ne!(std::env::consts::OS,"macos");
}
#[test]
fn reference_model_external_io_still_denied() {
    let (mut model,permit)=Model::bootstrap(0x4c,
        ScopeBudget{tasks:0,resources:0,children:0},16);
    let root=model.root();
    assert_eq!(model.host_grant(&permit,root,ResourceEffect::ExternalIo),
               Err(ModelError::UnsupportedEffect));
    assert_eq!(model.event_count(),0);
}
