use AppId;
use AppSlice;
use Callback;
use Shared;

pub trait Driver {
    fn subscribe(&self, subscribe_type: usize, callback: Callback) -> isize {
        -1
    }

    fn command(&self, cmd_type: usize, r2: usize) -> isize {
        -1
    }

    #[allow(unused)]
    fn allow(&self, app: AppId, allow_type: usize, slice: AppSlice<Shared, u8>) -> isize {
        -1
    }
}
