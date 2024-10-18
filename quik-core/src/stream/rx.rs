use std::future::Future;
use std::io::Read;

use quik_util::*;

pub trait StreamRx {
  fn on_data(&self, data: &[u8]) -> impl Future<Output = Result<()>>;
  fn on_close(&self) -> impl Future<Output = Result<()>>;
}

struct DefaultStreamRx {
  inner: Mutex<ReadStreamInner>,
}

impl DefaultStreamRx {
  fn new(max_len: Option<usize>) -> Self {
    Self {
      inner: Mutex::new(ReadStreamInner {
        max_len,
        data: Vec::new(),
        eof: false,
      }),
    }
  }
}

struct ReadStreamInner {
  max_len: Option<usize>,
  data: Vec<u8>,
  eof: bool,
}

impl StreamRx for DefaultStreamRx {
  async fn on_data(&self, data: &[u8]) -> Result<()> {
    let mut inner = self.inner.lock().await;
    if let Some(max_len) = inner.max_len {
      if inner.data.len() + data.len() > max_len {
        return Err("Data exceeds max length".into());
      }
    }
    inner.data.extend_from_slice(data);
    Ok(())
  }
  async fn on_close(&self) -> Result<()> {
    self.inner.lock().await.eof = true;
    Ok(())
  }
}

impl DefaultStreamRx {
  async fn read_async(&self, buf: &mut [u8]) -> std::io::Result<usize> {
    let mut inner = self.inner.lock().await;
    if inner.eof {
      return Ok(0);
    }

    let mut data = inner.data.as_slice();
    let len = data.read(buf)?;
    if len == inner.data.len() {
      inner.data.clear();
    } else if len != 0 {
      inner.data = data.to_vec();
    }
    Ok(len)
  }
}
