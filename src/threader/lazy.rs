use crossbeam::channel::{bounded, Receiver, Sender, TryRecvError};

#[derive(Debug)]
pub enum Lazy<T> {
    Val(Box<T>),
    Waiting(Receiver<Box<T>>),
}
impl<T> Lazy<T> {
    pub fn open() -> (Self, Sender<Box<T>>) {
        let (s, r) = bounded(1);
        (Self::Waiting(r), s)
    }
    pub fn val(val: T) -> Self {
        Self::Val(Box::new(val))
    }
    pub fn get(&mut self) -> &T {
        match self {
            Self::Val(ref val) => val,
            Self::Waiting(recv) => {
                *self = Lazy::Val(recv.recv().unwrap());
                if let Self::Val(val) = self {
                    val
                } else {
                    unreachable!()
                }
            }
        }
    }
    pub fn try_get(&mut self) -> Option<&T> {
        match self {
            Self::Val(ref val) => Some(val),
            Self::Waiting(recv) => {
                let val = match recv.try_recv() {
                    Ok(val) => val,
                    Err(TryRecvError::Empty) => return None,
                    _ => unreachable!(),
                };
                *self = Lazy::Val(val);
                if let Self::Val(val) = self {
                    Some(val)
                } else {
                    unreachable!()
                }
            }
        }
    }
    pub fn poll(&mut self) -> bool {
        if let Self::Waiting(recv) = self {
            if let Ok(val) = recv.try_recv() {
                *self = Lazy::Val(val);
            } else {
                return false;
            }
        }
        true
    }
}
