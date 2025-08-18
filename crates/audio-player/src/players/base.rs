use std::sync::Arc;
use types::errors::Result;
use types::ui::player_details::PlayerEvents;
use types::songs::{Song, SongType};
use tokio::sync::oneshot::Sender as OneShotSender;
use dyn_clone::DynClone;
use std::any::Any;

pub type PlayerEventsSender = Arc<dyn Fn(String, PlayerEvents) + Send + Sync>;

pub trait BasePlayer: std::fmt::Debug + DynClone + Send + Sync {
  fn initialize(&self);
  fn key(&self) -> String;
  fn load(&self, src: String, autoplay: bool, resolver: OneShotSender<()>);
  fn stop(&mut self) -> Result<()>;
  fn play(&self) -> Result<()>;
  fn pause(&self) -> Result<()>;
  fn seek(&self, pos: f64) -> Result<()>;
  fn provides(&self) -> &[SongType];
  fn can_play(&self, song: &Song) -> bool;
  fn set_volume(&self, volume: f64) -> Result<()>;
  fn get_volume(&self) -> Result<f64>;
  fn add_listeners(&mut self, state_setter: PlayerEventsSender);
  fn configure(&mut self, _key: &str, _opaque: &dyn Any) { }
}