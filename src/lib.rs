use std::{
  sync::{
    mpsc::{channel, Receiver, Sender},
    Arc,
  },
  thread,
};

use command_handler::handle_command;
use napi::{
  threadsafe_function::{ErrorStrategy, ThreadsafeFunction, ThreadsafeFunctionCallMode},
  Env, Error, JsFunction, JsObject, Result, Status,
};
use rodio::{decoder::DecoderInfo, OutputStream, Sink};

mod command_handler;
mod errors;

#[macro_use]
extern crate napi_derive;

type CommandReceiver = Receiver<(String, String, ThreadsafeFunction<f64>)>;

fn handle_audio_thread(receiver: CommandReceiver) -> CommandReceiver {
  let (_stream, stream_handle) = OutputStream::try_default().unwrap();
  let mut sink = Sink::try_new(&stream_handle).unwrap();
  let mut src: Option<Arc<DecoderInfo>> = None;

  loop {
    if let Ok((command, arg, tsfn)) = receiver.recv() {
      let res = handle_command(sink, command, arg, src.clone());
      match res {
        Ok((sink_tmp, source, ret)) => {
          sink = sink_tmp;

          if source.is_some() {
            src = source;
          }

          tsfn.call(
            Ok(ret.unwrap_or_default()),
            ThreadsafeFunctionCallMode::Blocking,
          );
        }
        Err(err) => {
          tsfn.call(
            Err(Error::new(Status::Unknown, err.to_string())),
            ThreadsafeFunctionCallMode::Blocking,
          );

          return receiver;
        }
      }
    }
  }
}

fn spawn_audio_thread(mut receiver: CommandReceiver) {
  thread::spawn(move || loop {
    receiver = handle_audio_thread(receiver);
  });
}

#[napi]
pub fn initialize(env: Env) -> Result<JsObject> {
  let mut ret = env.create_object()?;

  let (sender_ret, receiver_ret) = channel::<(String, String, ThreadsafeFunction<f64>)>();

  spawn_audio_thread(receiver_ret);

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
