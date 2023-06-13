use std::mem;

trait WaveGenerator {
    fn update<'a>(&'a mut self, buffer: &'a mut [f32]) -> &[f32];
}

struct SineWaveGenerator {
    frequency: u32,
    sampling_rate: u32,
    step: u32,
}

impl SineWaveGenerator {
    fn new(frequency: u32, sampling_rate: u32) -> Self {
        SineWaveGenerator {
            frequency,
            sampling_rate,
            step: 0,
        }
    }
}

impl WaveGenerator for SineWaveGenerator  {
    fn update<'a>(&'a mut self, buffer: &'a mut [f32]) -> &[f32]{
        use std::f32::consts::PI;
        let mut value = self.step as u32 * self.frequency as u32;
        let multiplier = PI * 2.0 / self.sampling_rate as f32;
        for sample in buffer.iter_mut() {
            *sample = f32::sin(value as f32 * multiplier);
            value += self.frequency;
        }
        self.step = (self.step + buffer.len() as u32) % self.sampling_rate;
        buffer
    }
}

/*
fn convert_to_u8<'a, T>(src: &'a[f32], dest: &'a mut [u8]) -> &'a [u8] {
    let dest_len = src.len() * mem::size_of::<f32>();
    assert!(dest_len <= dest.len());
    let mut i = 0;
    for s in src.iter() {
        for j in s.to_ne_bytes().iter() {
            dest[i] = *j;
            i += 1;
        } 
    }
    dest
}
*/

 /*
fn convert_to_u8_unsafe_ptr<'a, T>(src: &'a[T], dest: &'a mut [u8]) -> &'a [u8] {
    let mut dest_len = src.len() * mem::size_of::<T>();
    assert!(dest_len <= dest.len());
    unsafe {
        let mut psrc = src.as_ptr() as *const u8;
        let mut pdest = dest.as_mut_ptr() as *mut u8;
        while dest_len != 0 {
            *pdest = *psrc;
            psrc = psrc.offset(1);
            pdest = pdest.offset(1);
            dest_len -= 1;
        }
    }
    dest
}
*/

fn convert_to_u8_unsafe_memcpy<'a, T>(src: &'a[T], dest: &'a mut [u8]) -> &'a [u8] {
    let dst_len = src.len() * mem::size_of::<T>();
    assert!(dst_len <= dest.len());
    unsafe {
        use libc::c_void;
        libc::memcpy(dest.as_mut_ptr() as *mut c_void, src.as_ptr() as *const c_void, dst_len);
    }
    dest
}

fn main() {
    const SINE_WAVE_FREQUENCY: u32 = 440;
    const SAMPLING_RATE: u32 = 44100;

    use libpulse_binding::sample;
    use libpulse_binding::stream::Direction;
    use libpulse_simple_binding::Simple;

    let simple = Simple::new(
        None,
        "rust-pulseaudio-sine-wave",
        Direction::Playback,
        None,
        "sine",
        &sample::Spec {
            format: sample::Format::FLOAT32NE,
            channels: 1,
            rate: SAMPLING_RATE
        },
        None,
        None
    ).unwrap();

    println!("Sampling rate: {SAMPLING_RATE}Hz");
    println!("Sine wave frequency: {SINE_WAVE_FREQUENCY}Hz");
    let mut generator = SineWaveGenerator::new(SINE_WAVE_FREQUENCY, SAMPLING_RATE);

    let buffer_length = (SAMPLING_RATE / 100) as usize;
    println!("Sample buffer_length: {buffer_length}");
    let mut generate_buffer: Vec<f32> = vec![0_f32; buffer_length as usize];
    let mut convert_buffer = vec![0_u8; buffer_length * mem::size_of::<f32>()];

    loop {
        simple.write(convert_to_u8_unsafe_memcpy(generator.update(&mut generate_buffer), &mut convert_buffer)).unwrap();
    }
}
