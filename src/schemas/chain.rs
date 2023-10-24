use reqwest_eventsource::Event;
use tokio::sync::mpsc;

pub enum ChainResponse {
    Text(String),
    Stream(mpsc::Receiver<Result<Event, reqwest_eventsource::Error>>),
}
