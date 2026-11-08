//! G0.3 R3-R12 — in-process, engineering-only diagnostic probe.
//! Measures API call boundaries, not instruction cycle cost or language superiority.
//! No dependencies; never changes NORDOI's parser, NAIR encoding, or authority.
use std::hint::black_box;
use std::time::Instant;
use std::env;
use nordoi_kernel::{AtomicKernel, Capability, Instruction, NairProgram, RegisterId, Value, execute_nair};

const SCHEMA: &str = "NORDOI-G03-R3-R13-PROBE-v1";
const CASES: [&str; 7] = ["noop_set", "changed_set_flush", "dependency_set_flush", "noop_transaction", "denied_capability", "nair_validate", "nair_execute_with_validation"];

fn probe_case(name: &str, iterations: u64) -> Result<u128, &'static str> {
    let mut k = AtomicKernel::new();
    let src = k.create_atom(0_i64);
    let dep = k.create_atom(0_i64);
    if name == "dependency_set_flush" { k.connect(src, dep).map_err(|_| "connect failed")?; }
    let prog = NairProgram::from_instructions(vec![
        Instruction::Const { dst: RegisterId(0), value: Value::Int(42) },
        Instruction::Halt,
    ]);
    if name == "nair_execute_with_validation" { prog.validate().map_err(|_| "valid NAIR rejected")?; }
    let authority = Capability::ConsoleWrite;
    let start = Instant::now();
    for n in 0..iterations {
        match name {
            "noop_set" => {
                // Repeatedly assigning the same value must do no version or work mutation.
                let changed = k.set(src, black_box(0_i64)).map_err(|_| "noop write error")?;
                if changed { return Err("noop produced change"); }
                black_box(changed);
            }
            "changed_set_flush" | "dependency_set_flush" => {
                // Alternating values prevents dead-store measurement and forces a real update.
                let next = black_box((n % 2) as i64 + 1);
                let changed = k.set(src, next).map_err(|_| "changed write error")?;
                if !changed { return Err("changed write was skipped"); }
                let frontier = k.flush();
                let expected = if name == "dependency_set_flush" { 2 } else { 1 };
                if frontier.len() != expected { return Err("incorrect invalidation frontier"); }
                black_box(frontier);
            }
            "noop_transaction" => {
                let mut tx = k.begin_transaction(k.root_domain()).map_err(|_| "begin transaction failed")?;
                tx.set(&k, src, black_box(0_i64)).map_err(|_| "stage failed")?;
                let outcome = k.commit(tx).map_err(|_| "no-op commit failed")?;
                if outcome.changed_atoms != 0 || outcome.scheduled_atoms != 0 { return Err("no-op commit created work"); }
                black_box(outcome);
            }
            "denied_capability" => {
                if k.require(black_box(&authority)).is_ok() { return Err("ambient capability granted") }
            }
            "nair_validate" => {
                // Pure validity checking, separated from execute_nair (which revalidates).
                black_box(&prog).validate().map_err(|_| "NAIR validation failed")?;
            }
            "nair_execute_with_validation" => {
                // execute_nair revalidates the program on each call. This case includes validation.
                let outcome = execute_nair(&mut k, black_box(&prog)).map_err(|_| "valid NAIR failed")?;
                if outcome.executed_instructions != 2 || outcome.created_atoms != 0 || outcome.scheduled_work != 0 {
                    return Err("NAIR observable work mismatch");
                }
                black_box(outcome);
            }
            _ => return Err("unknown case"),
        }
    }
    let ns = start.elapsed().as_nanos();
    if name == "noop_set" || name == "noop_transaction" || name == "denied_capability" || name == "nair_execute_with_validation" {
        if k.version(src).map_err(|_| "missing atom")? != 0 || k.pending_work() != 0 {return Err("unexpected state mutation")}
    } else if k.version(src).map_err(|_| "missing atom")? != iterations || k.pending_work() != 0 {
        return Err("state-version / flush invariant failed");
    }
    Ok(ns)
}

fn median(values: &mut [u128]) -> u128 {
    values.sort_unstable();
    let len = values.len();
    if len % 2 == 1 {values[len / 2]} else {(values[len/2-1] + values[len/2])/2}
}

fn measure(iterations: u64, blocks: usize) -> Result<(), &'static str> {
    if !(100..=1_000_000).contains(&iterations) || !(3..=30).contains(&blocks) {return Err("invalid benchmark bounds")}
    // Warmup every public API boundary before timing, without counting these in the report.
    for name in CASES {probe_case(name, 100)?;}
    let mut observations: Vec<Vec<u128>> = vec![Vec::new(); CASES.len()];
    for block in 0..blocks {
        // Rotate order. Each probe_case starts with a fresh kernel, outside timed region.
        for offset in 0..CASES.len() {
            let idx = (block + offset) % CASES.len();
            let ns = probe_case(CASES[idx], iterations)?;
            observations[idx].push(ns);
        }
    }
    print!("{{\"schema\":\"{}\",\"method\":\"IN_PROCESS_WALL_CLOCK_INCLUDING_API_VALIDATION_AND_CHECKS\",\"iterations\":{},\"blocks\":{},\"units\":\"nanoseconds_per_operation_integer_floor\",\"cases\":{{",SCHEMA, iterations, blocks);
    for (i, name) in CASES.iter().enumerate() {
        let vals: Vec<u128> = observations[i].iter().map(|total| total / u128::from(iterations)).collect();
        let mut sorted = vals.clone();
        let med = median(&mut sorted);
        if i != 0 {print!(",");}
        print!("\"{}\":{{\"median_ns_per_op\":{},\"samples_ns_per_op\":[",name,med);
        for (j, value) in vals.iter().enumerate() {
            if j != 0 {print!(",");}
            print!("{}",value);
        }
        print!("]}}");
    }
    println!("}},\"cli_process_included\":false,\"compiler_included\":false,\"nair_includes_validation\":true,\"cross_language_comparison\":\"NOT_MEASURED\",\"human_evaluation_gate\":\"CLOSED_NOT_REQUIRED\",\"production\":\"NOT_AUTHORIZED\"}}");
    Ok(())
}

fn selftest() -> Result<(), &'static str> {
    // Run real core operations and invalid NAIR; this is a structural functional gate,
    // not a speed claim and not a human study.
    for name in CASES {probe_case(name, 100)?;}
    let mut k = AtomicKernel::new();
    let invalid = NairProgram::from_instructions(vec![
        Instruction::Const { dst: RegisterId(0), value: Value::Int(1) },
        Instruction::Const { dst: RegisterId(0), value: Value::Int(2) },
        Instruction::Halt,
    ]);
    if execute_nair(&mut k, &invalid).is_ok() || k.pending_work() != 0 {return Err("invalid NAIR not rejected")}
    if median(&mut [9, 3, 7]) != 7 {return Err("median mismatch")}
    println!("G03_R3_R13_RUST_SELFTEST=PASS");
    println!("HUMAN_EVALUATION_GATE=CLOSED_NOT_REQUIRED");
    println!("PRODUCTION=NOT_AUTHORIZED");
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let result = match args.as_slice() {
        [cmd] if cmd == "selftest" => selftest(),
        [cmd, case, iters] if cmd == "one" => {
            if !CASES.contains(&case.as_str()) {
                Err("unknown benchmark case")
            } else {
                match iters.parse::<u64>() {
                    Ok(n) if (100..=1_000_000).contains(&n) => {
                        match probe_case(case, n) {
                            Ok(total_ns) => {
                                println!(r#"{{"schema":"{}","case":"{}","iterations":{},"total_ns":{},"invariants":"PASS"}}"#,
                                    SCHEMA, case, n, total_ns);
                                Ok(())
                            }
                            Err(e) => Err(e),
                        }
                    }
                    _ => Err("out-of-bounds iterations"),
                }
            }
        }
        [cmd, iters, blocks] if cmd == "measure" => {
            match (iters.parse::<u64>(), blocks.parse::<usize>()) {
                (Ok(iterations), Ok(blocks)) => measure(iterations, blocks),
                _ => Err("invalid numeric arguments"),
            }
        }
        _ => Err("usage: nordoi_atomic_probe_r312 selftest | measure <iterations> <blocks>"),
    };
    if let Err(reason) = result {
        eprintln!("G03_R3_R12_BLOCKED={}", reason);
        std::process::exit(2);
    }
}
