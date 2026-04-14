use anyhow::{anyhow, Context, Result};
use embedded_hal::delay::DelayNs;
use hx711::{Hx711, Mode};
use rppal::gpio::{Gpio, InputPin, OutputPin};
use std::env;
use std::hint::spin_loop;
use std::thread;
use std::time::{Duration, Instant};

const DEFAULT_DOUT_PIN: u8 = 5;
const DEFAULT_SCK_PIN: u8 = 6;
const DEFAULT_TARE_SAMPLES: usize = 20;
const DEFAULT_READ_SAMPLES: usize = 10;
const DEFAULT_POLL_DELAY_MS: u64 = 500;

#[derive(Clone, Copy)]
struct Config {
    dout_pin: u8,
    sck_pin: u8,
    mode: Mode,
    tare_samples: usize,
    read_samples: usize,
    calibration_factor: f64,
    poll_delay_ms: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            dout_pin: DEFAULT_DOUT_PIN,
            sck_pin: DEFAULT_SCK_PIN,
            mode: Mode::ChAGain128,
            tare_samples: DEFAULT_TARE_SAMPLES,
            read_samples: DEFAULT_READ_SAMPLES,
            calibration_factor: 1.0,
            poll_delay_ms: DEFAULT_POLL_DELAY_MS,
        }
    }
}

struct StdDelay;

impl DelayNs for StdDelay {
    fn delay_ns(&mut self, ns: u32) {
        let target = Duration::from_nanos(u64::from(ns));
        let start = Instant::now();
        while start.elapsed() < target {
            spin_loop();
        }
    }
}

fn main() -> Result<()> {
    let config = parse_args(env::args().skip(1))?;

    let gpio = Gpio::new().context("failed to open GPIO")?;
    let dout = gpio
        .get(config.dout_pin)
        .with_context(|| format!("failed to open DOUT pin {}", config.dout_pin))?
        .into_input();
    let pd_sck = gpio
        .get(config.sck_pin)
        .with_context(|| format!("failed to open SCK pin {}", config.sck_pin))?
        .into_output_low();

    let mut hx711 = Hx711::new(StdDelay, dout, pd_sck)
        .map_err(|error| anyhow!("failed to initialize HX711: {error:?}"))?;
    hx711
        .set_mode(config.mode)
        .map_err(|error| anyhow!("failed to set HX711 mode: {error:?}"))?;

    let offset = tare(&mut hx711, config.tare_samples, config.poll_delay_ms)?;
    println!(
        "HX711 ready. DOUT={}, SCK={}, mode={}, tare_offset={offset}",
        config.dout_pin,
        config.sck_pin,
        mode_label(config.mode)
    );

    loop {
        let raw = read_average(&mut hx711, config.read_samples, config.poll_delay_ms)?;
        let corrected = raw - offset;
        let weight = corrected as f64 / config.calibration_factor;

        println!("raw={raw} corrected={corrected} weight={weight:.3} units");
        thread::sleep(Duration::from_millis(config.poll_delay_ms));
    }
}

fn parse_args<I>(args: I) -> Result<Config>
where
    I: IntoIterator<Item = String>,
{
    let mut config = Config::default();
    let mut args = args.into_iter();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--dout" => {
                config.dout_pin = parse_u8_flag(&mut args, "--dout")?;
            }
            "--sck" => {
                config.sck_pin = parse_u8_flag(&mut args, "--sck")?;
            }
            "--tare-samples" => {
                config.tare_samples = parse_usize_flag(&mut args, "--tare-samples")?;
            }
            "--read-samples" => {
                config.read_samples = parse_usize_flag(&mut args, "--read-samples")?;
            }
            "--calibration-factor" => {
                config.calibration_factor = parse_f64_flag(&mut args, "--calibration-factor")?;
            }
            "--poll-delay-ms" => {
                config.poll_delay_ms = parse_u64_flag(&mut args, "--poll-delay-ms")?;
            }
            "--mode" => {
                let value = next_value(&mut args, "--mode")?;
                config.mode = parse_mode(&value)?;
            }
            "--help" | "-h" => {
                print_usage();
                std::process::exit(0);
            }
            other => return Err(anyhow!("unrecognized argument: {other}")),
        }
    }

    if config.tare_samples == 0 {
        return Err(anyhow!("--tare-samples must be greater than 0"));
    }

    if config.read_samples == 0 {
        return Err(anyhow!("--read-samples must be greater than 0"));
    }

    if config.calibration_factor == 0.0 {
        return Err(anyhow!("--calibration-factor must not be 0"));
    }

    Ok(config)
}

fn parse_mode(value: &str) -> Result<Mode> {
    match value.to_ascii_lowercase().as_str() {
        "a128" | "a" | "chagain128" => Ok(Mode::ChAGain128),
        "a64" | "chagain64" => Ok(Mode::ChAGain64),
        "b32" | "chbgain32" => Ok(Mode::ChBGain32),
        other => Err(anyhow!("invalid mode '{other}'. Use a128, a64, or b32")),
    }
}

fn mode_label(mode: Mode) -> &'static str {
    match mode {
        Mode::ChAGain128 => "a128",
        Mode::ChAGain64 => "a64",
        Mode::ChBGain32 => "b32",
    }
}

fn parse_u8_flag<I>(args: &mut I, flag: &str) -> Result<u8>
where
    I: Iterator<Item = String>,
{
    Ok(next_value(args, flag)?.parse::<u8>().with_context(|| format!("invalid value for {flag}"))?)
}

fn parse_usize_flag<I>(args: &mut I, flag: &str) -> Result<usize>
where
    I: Iterator<Item = String>,
{
    Ok(next_value(args, flag)?.parse::<usize>().with_context(|| format!("invalid value for {flag}"))?)
}

fn parse_u64_flag<I>(args: &mut I, flag: &str) -> Result<u64>
where
    I: Iterator<Item = String>,
{
    Ok(next_value(args, flag)?.parse::<u64>().with_context(|| format!("invalid value for {flag}"))?)
}

fn parse_f64_flag<I>(args: &mut I, flag: &str) -> Result<f64>
where
    I: Iterator<Item = String>,
{
    Ok(next_value(args, flag)?.parse::<f64>().with_context(|| format!("invalid value for {flag}"))?)
}

fn next_value<I>(args: &mut I, flag: &str) -> Result<String>
where
    I: Iterator<Item = String>,
{
    args.next().ok_or_else(|| anyhow!("missing value for {flag}"))
}

fn print_usage() {
    println!(
        "Usage: cargo run --bin hx711_read -- [options]\n\n\
Options:\n\
  --dout <pin>                BCM pin for HX711 DOUT (default: {DEFAULT_DOUT_PIN})\n\
  --sck <pin>                 BCM pin for HX711 PD_SCK (default: {DEFAULT_SCK_PIN})\n\
  --mode <a128|a64|b32>       HX711 gain/channel mode (default: a128)\n\
  --tare-samples <n>          Number of samples to average for tare (default: {DEFAULT_TARE_SAMPLES})\n\
  --read-samples <n>          Number of samples to average for each reading (default: {DEFAULT_READ_SAMPLES})\n\
  --calibration-factor <f>    Raw counts per unit; set this after calibration (default: 1.0)\n\
  --poll-delay-ms <ms>        Delay between displayed readings (default: {DEFAULT_POLL_DELAY_MS})\n\
  -h, --help                  Show this message"
    );
}

fn tare(
    hx711: &mut Hx711<StdDelay, InputPin, OutputPin>,
    samples: usize,
    poll_delay_ms: u64,
) -> Result<i32> {
    read_average(hx711, samples, poll_delay_ms)
}

fn read_average(
    hx711: &mut Hx711<StdDelay, InputPin, OutputPin>,
    samples: usize,
    poll_delay_ms: u64,
) -> Result<i32> {
    let mut total = 0i64;

    for _ in 0..samples {
        total += i64::from(read_raw(hx711, poll_delay_ms)?);
    }

    Ok((total / samples as i64) as i32)
}

fn read_raw(
    hx711: &mut Hx711<StdDelay, InputPin, OutputPin>,
    poll_delay_ms: u64,
) -> Result<i32> {
    loop {
        match hx711.retrieve() {
            Ok(value) => return Ok(value),
            Err(nb::Error::WouldBlock) => {
                thread::sleep(Duration::from_millis(poll_delay_ms.min(10)))
            }
            Err(nb::Error::Other(error)) => {
                return Err(anyhow!("HX711 read error: {error:?}"));
            }
        }
    }
}