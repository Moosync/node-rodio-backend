use std::{fs::File, io::BufReader, sync::Arc, time::Duration};

use rodio::{decoder::DecoderInfo, Decoder, Sink, Source};

use crate::errors::CommandError;

type CommandReturnValue =
  std::result::Result<(Sink, Option<Arc<DecoderInfo>>, Option<f64>), CommandError>;

fn set_src(sink: Sink, path: String) -> CommandReturnValue {
  let file = File::open(path)?;

  let reader = BufReader::new(file);
  let source = Decoder::new(BufReader::new(reader))?;

  sink.clear();
  let src = source.get_info();
  let total_duration = source.total_duration().unwrap().as_millis() as f64;
  sink.append(source);

  Ok((sink, Some(src), Some(total_duration)))
}

fn play(sink: Sink) -> CommandReturnValue {
  sink.play();
  Ok((sink, None, None))
}

fn pause(sink: Sink) -> CommandReturnValue {
  sink.pause();
  Ok((sink, None, None))
}

fn stop(sink: Sink) -> CommandReturnValue {
  sink.stop();
  Ok((sink, None, None))
}

fn set_volume(sink: Sink, arg: String) -> CommandReturnValue {
  let volume = arg.parse::<f32>()?;
  sink.set_volume(volume as f32);
  Ok((sink, None, None))
}

fn get_volume(sink: Sink) -> CommandReturnValue {
  let volume = sink.volume() as f64;
  Ok((sink, None, Some(volume)))
}

fn get_postion(sink: Sink, old_src: Option<Arc<DecoderInfo>>) -> CommandReturnValue {
  if old_src.is_some() {
    let duration = old_src.unwrap().elapsed_duration();
    if duration.is_some() {
      return Ok((sink, None, Some(duration.unwrap().as_millis() as f64)));
    }
  }

  Ok((sink, None, None))
}

fn seek(sink: Sink, arg: String) -> CommandReturnValue {
  let pos = arg.parse::<u64>()?;
  sink.try_seek(Duration::from_millis(pos))?;
  Ok((sink, None, None))
}

pub fn handle_command(
  sink: Sink,
  command: String,
  arg: String,
  old_src: Option<Arc<DecoderInfo>>,
) -> CommandReturnValue {
  match command.as_str() {
    "SET_SRC" => set_src(sink, arg),
    "PLAY" => play(sink),
    "PAUSE" => pause(sink),
    "STOP" => stop(sink),
    "SET_VOLUME" => set_volume(sink, arg),
    "GET_VOLUME" => get_volume(sink),
    "GET_POSITION" => get_postion(sink, old_src),
    "SEEK" => seek(sink, arg),
    &_ => Err(CommandError::InvalidArg("Invaild arguments".to_string())),
  }
}
