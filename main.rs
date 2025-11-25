extern crate libafl;

use std::path::PathBuf;

use libafl::{
    bolts::{current_nanos, rands::StdRand, tuples::tuple_list, AsSlice},
    corpus::{InMemoryCorpus, OnDiskCorpus, QueueCorpusScheduler},
    events::SimpleEventManager,
    executors::{inprocess::InProcessExecutor, ExitKind},
    feedbacks::{CrashFeedback, MapFeedbackState, MaxMapFeedback},
    fuzzer::{Fuzzer, StdFuzzer},
    generators::RandPrintablesGenerator,
    inputs::{BytesInput, HasTargetBytes},
    monitors::SimpleMonitor,
    mutators::scheduled::{havoc_mutations, StdScheduledMutator},
    observers::StdMapObserver,
    stages::mutational::StdMutationalStage,
    state::StdState,
};

// Manual "coverage" map (no instrumentation yet).
static mut SIGNALS: [u8; 16] = [0; 16];

fn signals_set(idx: usize) {
    unsafe { SIGNALS[idx] = 1 };
}

fn main() {
    // === 1) Harness: what we fuzz ===
    // Panics if input starts with "abc".
    let mut harness = |input: &BytesInput| {
        let target = input.target_bytes();
        let buf = target.as_slice();

        // Fake "coverage" by toggling entries in SIGNALS.
        signals_set(0);
        if buf.len() > 0 && buf[0] == b'a' {
            signals_set(1);
            if buf.len() > 1 && buf[1] == b'b' {
                signals_set(2);
                if buf.len() > 2 && buf[2] == b'c' {
                    panic!("=)");
                }
            }
        }

        ExitKind::Ok
    };

    // === 2) Monitor + Event Manager ===
    let mon = SimpleMonitor::new(|s| println!("{s}"));
    let mut mgr = SimpleEventManager::new(mon);

    // === 3) Observer over our fake coverage map ===
    let observer = StdMapObserver::new("signals", unsafe { &mut SIGNALS });

    // === 4) Feedbacks ===
    // Feedback state based on the observer.
    let feedback_state = MapFeedbackState::with_observer(&observer);

    // "How interesting is this input?" -> coverage-based novelty search.
    let feedback = MaxMapFeedback::new(&feedback_state, &observer);

    // Objective: did we crash? (panic)
    let objective = CrashFeedback::new();

    // === 5) State: RNG + corpora + feedback_state ===
    let mut state = StdState::new(
        // RNG
        StdRand::with_seed(current_nanos()),
        // In-memory corpus for interesting inputs
        InMemoryCorpus::new(),
        // On-disk corpus for solutions (crashes)
        OnDiskCorpus::new(PathBuf::from("./crashes")).unwrap(),
        // feedback state(s) stored inside the State
        tuple_list!(feedback_state),
    );

    // === 6) Scheduler + Fuzzer ===
    let scheduler = QueueCorpusScheduler::new();
    let mut fuzzer = StdFuzzer::new(scheduler, feedback, objective);

    // === 7) Executor: runs harness in-process with our observer ===
    let mut executor = InProcessExecutor::new(
        &mut harness,
        tuple_list!(observer),
        &mut fuzzer,
        &mut state,
        &mut mgr,
    )
    .expect("Failed to create the Executor");

    // === 8) Seed corpus: random printable inputs ===
    let mut generator = RandPrintablesGenerator::new(32); // max length 32
    state
        .generate_initial_inputs(&mut fuzzer, &mut executor, &mut generator, &mut mgr, 8)
        .expect("Failed to generate the initial corpus");

    // === 9) Mutational stage: AFL-style havoc mutator ===
    let mutator = StdScheduledMutator::new(havoc_mutations());
    let mut stages = tuple_list!(StdMutationalStage::new(mutator));

    // === 🔁 10) Main fuzzing loop ===
    fuzzer
        .fuzz_loop(&mut stages, &mut executor, &mut state, &mut mgr)
        .expect("Error in fuzzing loop");
}
