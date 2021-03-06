/*!
* Function of PortMidi Time.
*/
extern crate time;

use std::io::timer;
use std::time::duration;
use std::comm;
use std::comm::{channel, Sender, Receiver};

pub enum PtError {
    PtNoError = 0,         /* success */
    PtHostError = -10000,  /* a system-specific error occurred */
    PtAlreadyStarted,      /* cannot start timer because it is already started */
    PtAlreadyStopped,      /* cannot stop timer because it is already stopped */
    PtInsufficientMemory   /* memory could not be allocated */
}

/*
    Pt_Sleep() pauses, allowing other threads to run.

    duration is the length of the pause in ms. The true duration
    of the pause may be rounded to the nearest or next clock tick
    as determined by resolution in Pt_Start().
*/
pub fn pt_sleep(duration: i64)	{
	timer::sleep(duration::Duration::milliseconds(duration));
}

pub struct PtTimer	{
	channel: Sender<String>,
	started: bool,
	start_time: u64,
}

impl PtTimer	{

	/**
	    Pt_Start() starts a real-time service.

	    resolution is the timer resolution in ms. The time will advance every
	    resolution ms.

	    callback is a function pointer to be called every resolution ms.

	    userData is passed to callback as a parameter.

	    return value: timer always start
	*/
	pub fn pt_start<T:Send> (resolution : i64, user_data : T , callback: extern "Rust" fn(u64, &mut T)) -> PtTimer {
    let (newchan, newport): (Sender<String>, Receiver<String>) = channel();
    let ptimer = PtTimer {
    	channel: newchan,
    	started: true,
    	start_time: time::precise_time_ns(),
    };

    spawn (proc() {
			let mut timer = timer::Timer::new().unwrap();
			let periodic = timer.periodic(duration::Duration::milliseconds(resolution));
			let mut stop : bool = false;
			let starttime = time::precise_time_ns();
			let mut mutdata = user_data;
			loop {
		    periodic.recv();
		    let now = time::precise_time_ns();
		    callback((now - starttime) / 1000000, &mut mutdata);
		    match newport.try_recv() {
		    	Ok(ref message) => {
          	if *message == "stop".to_string()	{
          		stop = true;
          	}
		    	},
		    	Err(comm::Empty) => (),
		    	Err(comm::Disconnected) => {
            panic!("Action channel disconnect error.")
          }
      	}
      	if stop	{
      		break;
      	}
		  }
    });
    ptimer
	}

	/*
    Pt_Stop() stops the timer.

    return value:
    Upon success, returns ptNoError. See PtError for other values.
*/
	pub fn  pt_stop(&mut self)	{
	    self.channel.send("stop".to_string());
	    self.started = false;
	}

	/*
	    Pt_Started() returns true iff the timer is running.
	*/
	pub fn  pt_started(&self) -> bool	{
		self.started
	}

	/*
	    Pt_Time() returns the current time in ms.
	*/
	pub fn pt_time(&self) -> u64	{
	    let now = time::precise_time_ns();
	    (now - self.start_time) / 1000000
	}

}
