//! NORDOI G0.3 R4O. Research-only pinned public root and two-cursor witness.
//! A laboratory-owned fingerprint is not external production trust custody.
//! Both writable cursor files restored together remain a KNOWN vulnerability.
#![allow(dead_code)]
#[path="../src/resource_task_r02.rs"]
mod reference;
use reference::{Model,ModelError,ResourceEffect,ScopeBudget};

#[cfg(target_os="macos")]
mod native {
    use std::fs::{self,OpenOptions};
    use std::io::Write;
    use std::path::{Path,PathBuf};
    use std::process::{Command,Stdio};
    use std::sync::atomic::{AtomicU64,Ordering};

    static NEXT:AtomicU64=AtomicU64::new(1);
    const NS:&str="nordoi-r4o-checkpoint@eddarsglobal";
    const PRINCIPAL:&str="nordoi-r4o-owner";

    fn run(mut cmd:Command, bytes:Option<&[u8]>)->std::process::Output {
        if bytes.is_some(){cmd.stdin(Stdio::piped());}
        let mut child=cmd.stdout(Stdio::piped()).stderr(Stdio::piped())
            .spawn().expect("real installed Mac utility");
        if let Some(data)=bytes {
            child.stdin.take().unwrap().write_all(data).unwrap();
        }
        child.wait_with_output().unwrap()
    }
    fn key(path:&Path)->String {
        let raw=fs::read_to_string(path).unwrap();
        let parts:Vec<&str>=raw.split_whitespace().collect();
        assert!(parts.len()>=2 && parts[0]=="ssh-ed25519");
        assert!(parts[1].len()>=40 && parts[1].bytes().all(|b|
            b.is_ascii_alphanumeric() || b==b'+' || b==b'/' || b==b'='));
        format!("{} {}\n",parts[0],parts[1])
    }
    fn sha256(b:&[u8],dir:&Path)->String {
        let file=dir.join("sha_input");
        fs::write(&file,b).unwrap();
        let mut cmd=Command::new("/usr/bin/shasum");
        cmd.args(["-a","256"]).arg(file);
        let out=run(cmd,None);
        assert!(out.status.success());
        let text=String::from_utf8_lossy(&out.stdout);
        let digest=text.split_whitespace().next().unwrap().to_string();
        assert_eq!(digest.len(),64);
        assert!(digest.bytes().all(|c|c.is_ascii_hexdigit()));
        digest
    }
    fn valid_cdhash(s:&str)->bool {
        s.len()==40 && s.bytes().all(|b|
            b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    }
    #[derive(Clone,Debug,PartialEq,Eq)]
    pub struct Head {pub epoch:u64,pub seq:u64,pub cdhash:String}
    impl Head {
        pub fn encode(&self)->String {
            format!("NORDOI-R4O-HEAD-V1\nepoch={}\nseq={}\ncdhash={}\n",
                self.epoch,self.seq,self.cdhash)
        }
        fn parse(s:&str)->Option<Self> {
            let parts:Vec<_>=s.lines().collect();
            if parts.len()!=4 || parts[0]!="NORDOI-R4O-HEAD-V1" {return None;}
            let v=Self {
                epoch:parts[1].strip_prefix("epoch=")?.parse().ok()?,
                seq:parts[2].strip_prefix("seq=")?.parse().ok()?,
                cdhash:parts[3].strip_prefix("cdhash=")?.to_string()
            };
            (v.epoch>0 && v.seq>0 && valid_cdhash(&v.cdhash)
                && v.encode()==s).then_some(v)
        }
    }
    pub struct Signed {pub msg:PathBuf,pub sig:PathBuf}
    pub struct Lab {
        dir:PathBuf,signer:PathBuf,verifier:PathBuf,witness:PathBuf,
        root:PathBuf,rogue:PathBuf,pub owner_pin:String,
        pub code:PathBuf,pub evil_code:PathBuf,
        pub code_hash:String,pub evil_hash:String,
    }
    impl Lab {
        pub fn new()->Self {
            let id=NEXT.fetch_add(1,Ordering::Relaxed);
            let dir=std::env::temp_dir().join(
                format!("nordoi-r4o-{}-{id}",std::process::id()));
            fs::create_dir(&dir).unwrap();
            let dir=fs::canonicalize(dir).unwrap();
            let signer=dir.join("signer"); let verifier=dir.join("verifier");
            let witness=dir.join("witness");
            for d in [&signer,&verifier,&witness]{fs::create_dir(d).unwrap();}
            let root=signer.join("root");let rogue=signer.join("rogue");
            for (p,label) in [(&root,"r4o-root"),(&rogue,"r4o-rogue")] {
                let mut c=Command::new("/usr/bin/ssh-keygen");
                c.args(["-q","-t","ed25519","-N","","-C",label,"-f"]).arg(p);
                assert!(run(c,None).status.success());
            }
            let approved=key(&root.with_extension("pub"));
            // The expected digest is held separately from any candidate
            // root submitted to the verifier. The harness still controls
            // BOTH in this laboratory; there is no real out-of-band owner.
            let owner_pin=sha256(approved.as_bytes(),&dir);
            let code=dir.join("approved");let evil_code=dir.join("changed");
            let mut l=Self {
                dir,signer,verifier,witness,root,rogue,owner_pin,
                code,evil_code,code_hash:String::new(),evil_hash:String::new()
            };
            l.build_code();l
        }
        fn build_code(&mut self){
            for (text,path) in [("approved",&self.code),("changed",&self.evil_code)]{
                let src=self.dir.join(format!("{text}.c"));
                fs::write(&src,format!(
                    "#include <stdio.h>\nint main(void){{puts(\"{text}\");return 0;}}\n"
                )).unwrap();
                let mut cc=Command::new("/usr/bin/cc");
                cc.args(["-O0","-x","c"]).arg(src).arg("-o").arg(path);
                assert!(run(cc,None).status.success());
                let mut sign=Command::new("/usr/bin/codesign");
                sign.args(["--force","--sign","-","--identifier",
                    "org.eddarsglobal.nordoi.r4o.test"]).arg(path);
                assert!(run(sign,None).status.success());
            }
            self.code_hash=Self::measure(&self.code);
            self.evil_hash=Self::measure(&self.evil_code);
            assert_ne!(self.code_hash,self.evil_hash);
        }
        fn measure(binary:&Path)->String{
            let mut verify=Command::new("/usr/bin/codesign");
            verify.args(["--verify","--strict"]).arg(binary);
            assert!(run(verify,None).status.success());
            let mut show=Command::new("/usr/bin/codesign");
            show.args(["--display","--verbose=4"]).arg(binary);
            let out=run(show,None);assert!(out.status.success());
            let t=String::from_utf8_lossy(&out.stderr);
            let h=t.lines().find_map(|l|l.trim().strip_prefix("CDHash="))
                .expect("native CodeDirectory CDHash").to_ascii_lowercase();
            assert!(valid_cdhash(&h));h
        }
        pub fn root_key(&self)->String {key(&self.root.with_extension("pub"))}
        pub fn rogue_key(&self)->String {key(&self.rogue.with_extension("pub"))}
        pub fn anchor(&self)->PathBuf {self.verifier.join("allowed_root")}
        pub fn states(&self)->(PathBuf,PathBuf){
            (self.verifier.join("cursor"),self.witness.join("checkpoint"))
        }
        pub fn enroll(&self,candidate:&str,expected_digest:&str)
            ->Result<(),&'static str>{
            if expected_digest.len()!=64
                || !expected_digest.bytes().all(|b|b.is_ascii_hexdigit()){
                return Err("INVALID_OWNER_PIN");
            }
            if !candidate.starts_with("ssh-ed25519 ")
                || candidate.matches('\n').count()!=1
                || !candidate.ends_with('\n') {
                return Err("NONCANONICAL_ROOT");
            }
            if sha256(candidate.as_bytes(),&self.dir)!=expected_digest {
                return Err("ROOT_NOT_APPROVED_BY_PIN");
            }
            if self.anchor().exists(){return Err("ROOT_ALREADY_ENROLLED");}
            let mut file=OpenOptions::new().create_new(true).write(true)
                .open(self.anchor()).map_err(|_|"ROOT_NOT_EXCLUSIVE")?;
            file.write_all(format!("{PRINCIPAL} {candidate}").as_bytes())
                .map_err(|_|"ROOT_WRITE_FAILED")?;
            file.sync_all().map_err(|_|"ROOT_SYNC_FAILED")?;
            Ok(())
        }
        pub fn enroll_root(&self){
            self.enroll(&self.root_key(),&self.owner_pin).unwrap();
        }
        pub fn sign(&self,id:&str,head:&Head,rogue:bool,ns:&str)->Signed {
            assert!(id.bytes().all(|b|b.is_ascii_alphanumeric()||b==b'-'));
            let msg=self.signer.join(format!("{id}.txt"));
            fs::write(&msg,head.encode()).unwrap();
            let mut cmd=Command::new("/usr/bin/ssh-keygen");
            cmd.args(["-Y","sign","-f"])
                .arg(if rogue{&self.rogue}else{&self.root})
                .args(["-n",ns]).arg(&msg);
            assert!(run(cmd,None).status.success());
            let sig=PathBuf::from(format!("{}.sig",msg.display()));
            assert!(sig.is_file());Signed{msg,sig}
        }
        pub fn signed_v1(&self,id:&str)->Signed{
            self.sign(id,&Head{epoch:1,seq:1,cdhash:self.code_hash.clone()},
                false,NS)
        }
        pub fn signed_v2(&self,id:&str)->Signed{
            self.sign(id,&Head{epoch:2,seq:2,cdhash:self.code_hash.clone()},
                false,NS)
        }
        fn check_anchor(&self,pin:&str)->Result<(),&'static str>{
            let text=fs::read_to_string(self.anchor())
                .map_err(|_|"ROOT_MISSING")?;
            let k=text.strip_prefix(&format!("{PRINCIPAL} "))
                .ok_or("ROOT_FORMAT_INVALID")?;
            if !k.starts_with("ssh-ed25519 ") || k.matches('\n').count()!=1 {
                return Err("ROOT_FORMAT_INVALID");
            }
            if sha256(k.as_bytes(),&self.dir)!=pin{
                return Err("ROOT_SUBSTITUTED");
            }
            Ok(())
        }
        fn check_sig(&self,bundle:&Signed)->bool{
            let data=match fs::read(&bundle.msg) {
                Ok(x) if x.len()<512=>x,_=>return false
            };
            let mut cmd=Command::new("/usr/bin/ssh-keygen");
            cmd.args(["-Y","verify","-f"]).arg(self.anchor())
                .args(["-I",PRINCIPAL,"-n",NS,"-s"]).arg(&bundle.sig);
            run(cmd,Some(&data)).status.success()
        }
        fn read_state(&self)->Result<Option<Head>,&'static str>{
            let (a,b)=self.states();
            if !a.exists() && !b.exists(){return Ok(None);}
            if !a.exists() || !b.exists(){return Err("ONE_WITNESS_MISSING");}
            let left=Head::parse(&fs::read_to_string(a)
                .map_err(|_|"LOCAL_UNREADABLE")?).ok_or("LOCAL_CORRUPT")?;
            let right=Head::parse(&fs::read_to_string(b)
                .map_err(|_|"WITNESS_UNREADABLE")?).ok_or("WITNESS_CORRUPT")?;
            if left!=right{return Err("WITNESSES_DIVERGED");}
            Ok(Some(left))
        }
        pub fn verify(&self,doc:&Signed,pin:&str,binary:&Path)
            ->Result<Head,&'static str>{
            self.check_anchor(pin)?;
            let cursor=self.read_state()?;
            if !self.check_sig(doc){return Err("ROOT_SIGNATURE_INVALID");}
            let raw=fs::read_to_string(&doc.msg).map_err(|_|"DOC_UNREADABLE")?;
            let h=Head::parse(&raw).ok_or("DOC_NONCANONICAL")?;
            if Self::measure(binary)!=h.cdhash{
                return Err("ACTUAL_CODE_MISMATCH");
            }
            match cursor {
                None if h.epoch==1 && h.seq==1 => Ok(h),
                None=>Err("FIRST_EPOCH_NOT_ONE"),
                Some(prev) if prev==h =>Ok(h),
                Some(prev) if h.epoch==prev.epoch+1 && h.seq>prev.seq =>Ok(h),
                _=>Err("EPOCH_OR_SEQUENCE_ROLLBACK"),
            }
        }
        fn store(path:&Path,head:&Head){
            let tmp=path.with_extension("next");
            let mut f=OpenOptions::new().create_new(true).write(true)
                .open(&tmp).unwrap();
            f.write_all(head.encode().as_bytes()).unwrap();
            f.sync_all().unwrap();
            fs::rename(tmp,path).unwrap();
        }
        pub fn commit(&self,h:&Head){
            let (a,b)=self.states();
            Self::store(&a,h);Self::store(&b,h);
        }
    }
    impl Drop for Lab {
        fn drop(&mut self){fs::remove_dir_all(&self.dir).unwrap();}
    }

    #[test]
    fn macos_owner_pin_required_and_second_enrollment_refused(){
        let x=Lab::new();
        assert_ne!(x.root_key(),x.rogue_key());
        assert_eq!(x.enroll(&x.rogue_key(),&x.owner_pin),
            Err("ROOT_NOT_APPROVED_BY_PIN"));
        x.enroll_root();
        assert_eq!(x.enroll(&x.root_key(),&x.owner_pin),
            Err("ROOT_ALREADY_ENROLLED"));
        assert_eq!(x.verify(&x.signed_v1("genesis"),&x.owner_pin,&x.code)
            .unwrap().seq,1);
    }
    #[test]
    fn macos_substituted_root_with_valid_rogue_signature_denied(){
        let x=Lab::new();x.enroll_root();
        let fake=x.sign("rogue",&Head{
            epoch:1,seq:1,cdhash:x.code_hash.clone()
        },true,NS);
        fs::write(x.anchor(),format!("{PRINCIPAL} {}",x.rogue_key())).unwrap();
        assert_eq!(x.verify(&fake,&x.owner_pin,&x.code),Err("ROOT_SUBSTITUTED"));
    }
    #[test]
    fn macos_cross_namespace_and_tampered_bytes_denied(){
        let x=Lab::new();x.enroll_root();
        let domain=x.sign("cross",&Head{
            epoch:1,seq:1,cdhash:x.code_hash.clone()
        },false,"nordoi-r4o-other@eddarsglobal");
        assert_eq!(x.verify(&domain,&x.owner_pin,&x.code),
            Err("ROOT_SIGNATURE_INVALID"));
        let alter=x.signed_v1("alter");
        fs::write(&alter.msg,Head{
            epoch:3,seq:3,cdhash:x.code_hash.clone()
        }.encode()).unwrap();
        assert_eq!(x.verify(&alter,&x.owner_pin,&x.code),
            Err("ROOT_SIGNATURE_INVALID"));
    }
    #[test]
    fn macos_observed_macho_code_not_authorized_by_manifest(){
        let x=Lab::new();x.enroll_root();
        assert_ne!(x.code_hash,x.evil_hash);
        assert_eq!(x.verify(&x.signed_v1("good"),&x.owner_pin,&x.evil_code),
            Err("ACTUAL_CODE_MISMATCH"));
    }
    #[test]
    fn macos_local_only_rollback_detected_by_unmodified_witness(){
        let x=Lab::new();x.enroll_root();
        let one=x.signed_v1("v1");
        let old=x.verify(&one,&x.owner_pin,&x.code).unwrap();x.commit(&old);
        let two=x.signed_v2("v2");
        let new=x.verify(&two,&x.owner_pin,&x.code).unwrap();x.commit(&new);
        let (local,_)=x.states();
        fs::write(local,old.encode()).unwrap();
        assert_eq!(x.verify(&one,&x.owner_pin,&x.code),
            Err("WITNESSES_DIVERGED"));
    }
    #[test]
    fn macos_witness_only_rollback_detected_by_unmodified_local(){
        let x=Lab::new();x.enroll_root();
        let one=x.signed_v1("v1");
        let old=x.verify(&one,&x.owner_pin,&x.code).unwrap();x.commit(&old);
        let two=x.signed_v2("v2");
        let new=x.verify(&two,&x.owner_pin,&x.code).unwrap();x.commit(&new);
        let (_,witness)=x.states();
        fs::write(witness,old.encode()).unwrap();
        assert_eq!(x.verify(&two,&x.owner_pin,&x.code),
            Err("WITNESSES_DIVERGED"));
    }
    #[test]
    fn macos_missing_or_corrupt_external_witness_fail_closed(){
        let x=Lab::new();x.enroll_root();
        let one=x.signed_v1("one");
        let old=x.verify(&one,&x.owner_pin,&x.code).unwrap();x.commit(&old);
        let (_,witness)=x.states();
        fs::remove_file(&witness).unwrap();
        assert_eq!(x.verify(&one,&x.owner_pin,&x.code),
            Err("ONE_WITNESS_MISSING"));
        fs::write(witness,"corrupt").unwrap();
        assert_eq!(x.verify(&one,&x.owner_pin,&x.code),
            Err("WITNESS_CORRUPT"));
    }
    #[test]
    fn macos_signed_prior_epoch_rejected_after_advance(){
        let x=Lab::new();x.enroll_root();
        let one=x.signed_v1("v1");
        let old=x.verify(&one,&x.owner_pin,&x.code).unwrap();x.commit(&old);
        let two=x.signed_v2("v2");
        let newer=x.verify(&two,&x.owner_pin,&x.code).unwrap();x.commit(&newer);
        assert_eq!(x.verify(&one,&x.owner_pin,&x.code),
            Err("EPOCH_OR_SEQUENCE_ROLLBACK"));
        assert_eq!(x.verify(&two,&x.owner_pin,&x.code),Ok(newer));
    }
    #[test]
    fn macos_restoring_both_writable_copies_is_known_undetected_attack(){
        let x=Lab::new();x.enroll_root();
        let one=x.signed_v1("v1");
        let old=x.verify(&one,&x.owner_pin,&x.code).unwrap();x.commit(&old);
        let two=x.signed_v2("v2");
        let newer=x.verify(&two,&x.owner_pin,&x.code).unwrap();x.commit(&newer);
        let (local,witness)=x.states();
        fs::write(local,old.encode()).unwrap();
        fs::write(witness,old.encode()).unwrap();
        // Counterexample: the writable "external" witness is not genuinely
        // independent of an attacker with write access to both directories.
        assert_eq!(x.verify(&one,&x.owner_pin,&x.code),Ok(old));
    }
}
#[cfg(not(target_os="macos"))]
#[test]
fn non_macos_never_claims_native_root_custody_or_monotonic_witness(){
    assert_ne!(std::env::consts::OS,"macos");
}
#[test]
fn reference_external_io_is_still_unsupported(){
    let (mut model,permit)=Model::bootstrap(
        0x4f,ScopeBudget{tasks:0,resources:0,children:0},16);
    let root=model.root();
    assert_eq!(model.host_grant(&permit,root,ResourceEffect::ExternalIo),
        Err(ModelError::UnsupportedEffect));
    assert_eq!(model.event_count(),0);
}
