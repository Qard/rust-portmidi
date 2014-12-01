#![crate_name = "portmidi"]
#![comment = "PortMidi binding for Rust"]
#![license = "MIT"]
#![crate_type = "lib"]
#![crate_type = "dylib"]

extern crate libc;
extern crate core;
extern crate serialize;

#[allow(non_camel_case_types)]
pub mod midi;
pub mod time;
#[allow(non_camel_case_types)]
pub mod util;

#[allow(type_overflow)]
#[cfg(test)]
mod tests {
    use {midi, time, util};
    use midi::PmError;

    struct Sequencer   {
        midi_notes: Vec<midi::PmEvent>,
        inport : Box<midi::PmInputPort>,
    }

    fn sequencer_callback(time:u64, data:&mut Sequencer)   {
        println!("sequencer_callback time:{}", time);
     //   let inport = *data.inport;
        let _ = data.midi_notes; // avoid dead code warning

        while match data.inport.poll() { PmError::PmGotData => true, _ => false }    {
            // println!("portmidi input note {}", readMidi);
            match data.inport.read()    {
                Ok(notes) => println!("portmidi read midi note {}", notes),
                Err(PmError::PmNoError) => println!("portmidi read midi no note {}", PmError::PmNoError),
                Err(err) => println!("portmidi read midi error {}", err)
            }
        }

    }


    #[test]
    fn test_midiin() {
    	let error:midi::PmError = midi::initialize();
    	assert_eq!(error as int, PmError::PmNoError as int);

    	let nbdevice : int = midi::count_devices();
    	println!("portmidi nb device {}", nbdevice);
    	let defdevin : int = midi::get_default_input_device_id();
    	println!("portmidi default input device {}", defdevin);
    	let defdevout : int = midi::get_default_output_device_id();
    	println!("portmidi default output device {}", defdevout);

        let ininfo = midi::get_device_info(defdevin);
        println!("portmidi default input device info {}", ininfo);

        let outinfo = midi::get_device_info(defdevout);
        println!("portmidi default output device info {}", outinfo);

        let mut inport : midi::PmInputPort = midi::PmInputPort::new(defdevin, 0);
        let inerror = inport.open();
        assert_eq!(inerror as int, PmError::PmNoError as int);

        let mut outport : midi::PmOutputPort = midi::PmOutputPort::new(defdevout, 100);
        let outerror = outport.open();
        println!("portmidi new output device {}", outerror);
        assert_eq!(outerror as int, PmError::PmNoError as int);

        let read_midi = inport.read();
        println!("portmidi input note {}", read_midi);
        match read_midi    {
            Ok(notes) => println!("portmidi read midi note {}", notes),
            Err(PmError::PmNoError) => println!("portmidi read midi no note {}", PmError::PmNoError),
            Err(err) => println!("portmidi read midi error {}", err)
        }

        let innote = inport.poll();
        assert_eq!(innote as int, PmError::PmNoError as int);

        //send note
        let note1 = midi::PmEvent {
            message : midi::PmMessage {
                status : 1 | 0x90, //chanell and note on
                data1 : 36, //note number
                data2 : 90, // velocity
            },
            timestamp : 0
        };
        let sendnoteerr = outport.write_event(note1);
        assert_eq!(sendnoteerr as int, PmError::PmNoError as int);

        let note2 = midi::PmMessage {
            status : 1 | 0x80, //chanell and note off
            data1 : 36, //note number
            data2 : 0, // velocity
        };
        let sendnote2err = outport.write_message(note2);
        assert_eq!(sendnote2err as int, PmError::PmNoError as int);

        //test sequencer
        let data = Sequencer{midi_notes: vec!(), inport: box inport};
        let mut timer = time::PtTimer::pt_start(1000, data, sequencer_callback);
        time::pt_sleep(10000);
        timer.pt_stop();



        //close out port
        let aborterr = outport.abort();
        assert_eq!(aborterr as int, PmError::PmNoError as int);
        let outcloseerr = outport.close();
        assert_eq!(outcloseerr as int, PmError::PmNoError as int);

        //close in port
        // let incloseerr = inport.close();
        // assert_eq!(incloseerr as int, PmError::PmNoError as int);

        //terminate midi
    	let error:midi::PmError = midi::terminate();
    	assert_eq!(error as int, PmError::PmNoError as int);
    }

   #[test]
    fn test_queue() {
        let mut queue : util::PmQueue = util::PmQueue::new();
        queue.create(32, 4);

        let read_midi = queue.dequeue();
        match read_midi    {
            Ok(notes) => println!("portmidi read midi note {}", notes),
            Err(PmError::PmNoError) => assert_eq!(PmError::PmNoError as int, PmError::PmNoError as int),
            Err(err) => panic!("portmidi read midi error {}", err)
        }

        assert_eq!(queue.is_empty(), true);
        assert_eq!(queue.is_full(), false);

        let peek1 = queue.peek();
        match peek1   {
            None => assert_eq!(peek1, None),
            _ => panic!("queue.peek  bad result. not None"),
        }

        let enqueuerr = queue.enqueue (
            midi::PmMessage {
                status : 1 | 0x90, //chanell and note on
                data1 : 36, //note number
                data2 : 90, // velocity
            }
        );
        assert_eq!(enqueuerr as int, PmError::PmNoError as int);

        assert_eq!(queue.is_empty(), false);
        assert_eq!(queue.is_full(), false);

        let peek1 = queue.peek();
        match peek1   {
            Some(notes) => assert_eq!(notes.data1, 36),
            None => panic!("queue.peek2  bad result. None"),
        }

        assert_eq!(queue.is_empty(), false);
        assert_eq!(queue.is_full(), false);

        let readqueue = queue.dequeue();
        match readqueue    {
            Ok(notes) => assert_eq!(notes.data1, 36),
            Err(PmError::PmNoError) => panic!("dequeue error no object found {}", readqueue),
            Err(err) => panic!("portmidi read midi error {}", err)
        }

        assert_eq!(queue.is_empty(), true);
        assert_eq!(queue.is_full(), false);

        let queudesterr = queue.destroy();
        assert_eq!(queudesterr as int, PmError::PmNoError as int);
   }

   #[deriving(Show)]
   struct TestMutCallback   {
        data: int,
   }

    fn test_callback(time:u64, data:&mut TestMutCallback)   {
        data.data = data.data + 1;
        println!("testcallback time:{} data:{}", time, data);
        let cal: int = (time / 1000) as int;
        assert_eq!(data.data, cal);
    }

    #[test]
    fn test_timer() {
        let data : TestMutCallback = TestMutCallback{data: 0};
        let mut timer = time::PtTimer::pt_start(1000, data, test_callback);
        assert_eq!(timer.pt_started(), true);

        println!("test_timer start time:{} ", timer.pt_time());

        time::pt_sleep(5000);
        timer.pt_stop();
        assert_eq!(timer.pt_started(), false);
        println!("test_timer end time:{} ", timer.pt_time());
        assert_eq!(timer.pt_time() >= 5000, true);
        assert_eq!(timer.pt_time() < 6000, true);
        assert_eq!(data.data, 0);
    }
}
