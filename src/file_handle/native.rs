use std::{
    future::Future,
    path::{Path, PathBuf},
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
};

#[derive(Default)]
struct ReaderState {
    res: Option<std::io::Result<Vec<u8>>>,
    waker: Option<Waker>,
}

struct Reader {
    state: Arc<Mutex<ReaderState>>,
}

impl Reader {
    fn new(path: &Path) -> Self {
        let state: Arc<Mutex<ReaderState>> = Arc::new(Mutex::new(Default::default()));

        {
            let path = path.to_owned();
            let state = state.clone();
            std::thread::Builder::new()
                .name("rfd_file_read".into())
                .spawn(move || {
                    let res = std::fs::read(path);

                    let mut state = state.lock().unwrap();
                    state.res.replace(res);

                    if let Some(waker) = state.waker.take() {
                        waker.wake();
                    }
                })
                .unwrap();
        }

        Self { state }
    }
}

impl Future for Reader {
    type Output = Vec<u8>;

    fn poll(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut state = self.state.lock().unwrap();
        if state.res.is_some() {
            let res = state.res.take().unwrap();
            Poll::Ready(res.unwrap())
        } else {
            state.waker.replace(ctx.waker().clone());
            Poll::Pending
        }
    }
}

/// FileHandle is a way of abstracting over a file returned by a dialog
pub struct FileHandle(PathBuf);

impl FileHandle {
    /// On native platforms it wraps path.
    ///
    /// On `WASM32` it wraps JS `File` object.
    pub fn wrap(path_buf: PathBuf) -> Self {
        Self(path_buf)
    }

    /// Get name of a file
    pub fn file_name(&self) -> String {
        self.0
            .file_name()
            .and_then(|f| f.to_str())
            .map(|f| f.to_string())
            .unwrap_or_default()
    }

    /// Gets path to a file.
    ///
    /// Does not exist in `WASM32`
    pub fn path(&self) -> &Path {
        &self.0
    }

    /// Reads a file asynchronously.
    ///
    /// On native platforms it spawns a `std::thread` in the background.
    ///
    /// `This fn exists souly to keep native api in pair with async only web api.`
    pub async fn read(&self) -> Vec<u8> {
        Reader::new(&self.0).await
    }

    /// Unwraps a `FileHandle` and returns innet type.
    ///
    /// It should be used, if user wants to handle file read themselves
    ///
    /// On native platforms returns path.
    ///
    /// On `WASM32` it returns JS `File` object.
    ///
    /// #### Behind a `file-handle-inner` feature flag
    #[cfg(feature = "file-handle-inner")]
    pub fn inner(&self) -> &Path {
        &self.0
    }
}

impl std::fmt::Debug for FileHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.path())
    }
}
