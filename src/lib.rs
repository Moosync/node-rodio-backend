use core::fmt;
use std::{
  fs::File,
  io::BufReader,
  num::{ParseFloatError, ParseIntError},
  sync::{
    mpsc::{channel, Sender},
    Arc,
  },
  thread,
  time::Duration,
};

use napi::{
  threadsafe_function::{ErrorStrategy, ThreadsafeFunction, ThreadsafeFunctionCallMode},
  Env, Error, JsFunction, JsObject, Result, Status,
};
use rodio::{decoder::DecoderInfo, source::SeekError, Decoder, OutputStream, Sink, Source};

#[macro_use]
extern crate napi_derive;

#[derive(Debug)]
enum CommandError {
  ParseFloatError(ParseFloatError),
  ParseIntError(ParseIntError),
  SeekError(SeekError),
  InvalidArg(String),
}

impl From<ParseFloatError> for CommandError {
  fn from(value: ParseFloatError) -> Self {
    return CommandError::ParseFloatError(value);
  }
}

impl From<ParseIntError> for CommandError {
  fn from(value: ParseIntError) -> Self {
    return CommandError::ParseIntError(value);
  }
}

impl From<SeekError> for CommandError {
  fn from(value: SeekError) -> Self {
    return CommandError::SeekError(value);
  }
}

impl From<&str> for CommandError {
  fn from(value: &str) -> Self {
    return CommandError::InvalidArg(value.to_string());
  }
}

impl fmt::Display for CommandError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{:?}", self)
  }
}

fn handle_command(
  sink: Sink,
  command: String,
  arg: String,
  old_src: Option<Arc<DecoderInfo>>,
) -> (
  Sink,
  Option<CommandError>,
  Option<f64>,
  Option<Arc<DecoderInfo>>,
) {
  let mut err: Option<CommandError> = None;
  let mut ret: Option<f64> = None;
  let mut src: Option<Arc<DecoderInfo>> = None;

  match command.as_str() {
    "SET_SRC" => {
      let file = BufReader::new(File::open(arg.clone()).unwrap());
      let source = Decoder::new(BufReader::new(file)).unwrap();
      sink.clear();
      src = Some(source.get_info());
      ret = Some(source.total_duration().unwrap().as_millis() as f64);
      sink.append(source);
    }
    "PLAY" => sink.play(),
    "PAUSE" => sink.pause(),
    "STOP" => sink.stop(),
    "SET_VOLUME" => {
      let volume = arg.parse::<f32>();
      match volume {
        Ok(v) => sink.set_volume(v as f32),
        Err(e) => err = Some(CommandError::ParseFloatError(e)),
      }
    }
    "GET_VOLUME" => {
      ret = Some(sink.volume() as f64);
    }
    "GET_POSITION" => {
      if old_src.is_some() {
        let duration = old_src.unwrap().elapsed_duration();
        if duration.is_some() {
          ret = Some(duration.unwrap().as_millis() as f64);
        }
      }
    }
    "SEEK" => {
      let pos = arg.parse::<u64>();
      err = match pos {
        Ok(p) => {
          let res = sink.try_seek(Duration::from_millis(p));
          println!("Seeked {:?}", res);
          match res {
            Ok(_) => None,
            Err(e) => Some(CommandError::SeekError(e)),
          }
        }
        Err(e) => Some(CommandError::ParseIntError(e)),
      };
    }
    &_ => err = Some(CommandError::InvalidArg("Invalid argument".to_string())),
  }

  (sink, err, ret, src)
}

#[napi]
pub fn initialize(env: Env) -> Result<JsObject> {
  let mut ret = env.create_object()?;

  let (sender_ret, receiver_ret) = channel::<(String, String, ThreadsafeFunction<f64>)>();

  thread::spawn(move || {
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let mut sink = Sink::try_new(&stream_handle).unwrap();
    let mut src: Option<Arc<DecoderInfo>> = None;

    loop {
      if let Ok((command, arg, res)) = receiver_ret.try_recv() {
        println!("Calling callback");
        let (sink_tmp, err, ret, source) = handle_command(sink, command, arg, src.clone());
        sink = sink_tmp;

        if source.is_some() {
          println!("Setting src");
          src = source;
        }

        if err.is_some() {
          res.call(
            Err(Error::new(Status::Unknown, err.unwrap().to_string())),
            ThreadsafeFunctionCallMode::Blocking,
          );
        } else {
          res.call(
            Ok(ret.unwrap_or_default()),
            ThreadsafeFunctionCallMode::Blocking,
          );
        }
      }

      thread::sleep(Duration::from_millis(100))
    }
  });

  env.wrap(&mut ret, sender_ret)?;

  Ok(ret)
}

#[napi]
pub fn send_command(
  env: Env,
  command: String,
  arg: String,
  sender_obj: JsObject,
  callback: JsFunction,
) -> Result<()> {
  let sender: &Sender<(String, String, ThreadsafeFunction<f64>)> = env.unwrap(&sender_obj)?;

  let tsfn_callback: ThreadsafeFunction<f64, ErrorStrategy::CalleeHandled> =
    callback.create_threadsafe_function(0, |ctx| Ok(vec![ctx.value]))?;

  sender
    .send((command, arg, tsfn_callback))
    .map_err(|err| napi::Error::new(Status::Unknown, err.to_string()))
}
