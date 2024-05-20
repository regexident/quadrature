use quadrature_decoder::{FullStep, HalfStep, IndexedIncrementalDecoder, QuadStep, StepMode};

fn decode<S, I>(decoder: &mut IndexedIncrementalDecoder<S>, pulse_trains: I)
where
    S: StepMode,
    I: Iterator<Item = ((bool, bool), bool)>,
{
    println!("Decoder is at counter: {:?}.", decoder.counter());
    println!();

    for ((a, b), z) in pulse_trains {
        println!("Reading pulses: (a: {a:?}, b: {b:?}, z: {z:?})");
        match decoder.update(a, b, z) {
            Ok(Some(change)) => println!("Change detected: {:?}.", change),
            Ok(None) => println!("No change detected."),
            Err(error) => println!("Error detected: {:?}.", error),
        }
        println!("Decoder is at counter: {:?}.", decoder.counter());
        println!();
    }
}

fn main() {
    let a: Vec<bool> = vec![false, false, true, true, false, false, true, true];
    let b: Vec<bool> = vec![true, false, false, true, true, false, false, true];
    let z: Vec<bool> = vec![false, false, false, false, true, false, false, false];

    println!("Full-step decoder:");
    let mut full_step_decoder = IndexedIncrementalDecoder::<FullStep>::default();
    decode(
        &mut full_step_decoder,
        a.iter()
            .cloned()
            .zip(b.iter().cloned())
            .zip(z.iter().cloned()),
    );
    println!();

    println!("Half-step decoder:");
    let mut half_step_decoder = IndexedIncrementalDecoder::<HalfStep>::default();
    decode(
        &mut half_step_decoder,
        a.iter()
            .cloned()
            .zip(b.iter().cloned())
            .zip(z.iter().cloned()),
    );
    println!();

    println!("Quad-step decoder:");
    let mut quad_step_decoder = IndexedIncrementalDecoder::<QuadStep>::default();
    decode(
        &mut quad_step_decoder,
        a.iter()
            .cloned()
            .zip(b.iter().cloned())
            .zip(z.iter().cloned()),
    );
    println!();
}
