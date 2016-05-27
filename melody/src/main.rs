mod note;
mod scale_pitch;
mod interval;
//mod rythmic_interval;
mod tuning;
mod melody;
mod synth_event;
mod tagged_event;

extern crate synth_interface;
extern crate basic_synth;
extern crate sdl2;

use synth_event::SynthEvent;
use tagged_event::TaggedEvent;
use melody::Melody;
use tuning::Tuning;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::audio::{AudioCallback, AudioSpecDesired};

use synth_interface::{Synth, SynthFactory};


//use std::sync::mpsc;

use std::rc::Rc;

struct SynthPlayer {
    synth: Box<Synth>,
    event_queue: Vec<TaggedEvent<SynthEvent>>,
    time: u32,
    current_event: usize,
}


impl AudioCallback for SynthPlayer {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {

        let mut previous_frame: (f32, f32) = (0., 0.);

        for (i, x) in out.iter_mut().enumerate() {
            if i%2==0 {
                while self.event_queue[self.current_event].tag <= self.time {

                    match self.event_queue[self.current_event].event {
                        SynthEvent::On { note_id, ref note_params } => {
                            self.synth.note_on(note_id, 0, note_params);
                        }
                        SynthEvent::Off { note_id } => {
                            self.synth.note_off(note_id);
                        }
                    }
                    self.current_event+=1;
                    if self.current_event >= self.event_queue.len() {
                        self.current_event = 0;
                        self.time = 0;
                    }
                }
                            
                previous_frame = self.synth.get_audio_frame();
                *x=previous_frame.0;
                self.time+=1;
            }
            else {
                *x=previous_frame.1;
            }
        }
    }
}


fn main() {

    let scale = {
        let mut pre_scale = (1..8)
            .map(|i| ((i as f64) * 7./12.) % 1.)
            .collect::<Vec<f64>>();

        pre_scale.sort_by(|a,b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        pre_scale
    }
    .iter()
        .map(|note_pitch| tuning::ScaleNote {
            interval: interval::Interval::from_octaves(*note_pitch),
            name: "".to_string(),
        })
    .collect::<Vec<_>>();


    let accidentals = vec![(1./12.,"Half-Tone"), (1./24.,"Quarter-Tone") ]
        .iter()
        .map(|&(pitch, name)| tuning::Accidental {
            interval: interval::Interval::from_octaves(pitch),
            name: name.to_string(),
        })
        .collect::<Vec<_>>();

    
    let tuning = Tuning {
        scale: scale,
        accidentals: accidentals,
        reference_frequency: 440.,
        name: "Standard Western Tuning".to_string(),
    };

    let rced_tuning = Rc::new(tuning);
    
    let mut notes: Vec<note::Note> = Vec::with_capacity(24);
   
    let make_pitch = | i: isize, accidentals: Vec<i32> | -> scale_pitch::ScalePitch {
        scale_pitch::ScalePitch {
            tuning: rced_tuning.clone(),
            range: 0 + (i as i32 / 7),
            scale_degree: i as i32 % 7,
            accidentals_count: accidentals,
            adjustment: interval::Interval::from_octaves(0.),
        }
    };


    {

        let mut push_note = |j: isize, accidentals, start, duration| {
            notes.push( note::Note{
                start: start,
                duration: duration,
                pitch: make_pitch(j+2, accidentals),
                amplitude: 0.3, // who cares
            });
        };


        push_note(0, vec![0,0], 0., 2. );
        push_note(2, vec![0,0], 0., 2. );
        push_note(4, vec![0,0], 0., 2. );

        push_note(3+0, vec![0,0], 2., 1. );
        push_note(3+2, vec![0,0], 2., 1. );
        push_note(0, vec![0,0],   2., 1. );

        push_note(4+0, vec![0,0], 3., 1. );
        push_note(4+2, vec![1,0], 3., 1. );
        push_note(1, vec![0,0],   3., 1. );



    } 

    let mut melody = Melody {
        notes: notes
    };


    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let audio_subsystem = sdl_context.audio().unwrap();

    let desired_spec = AudioSpecDesired {
        freq: Some(44100),
        channels: Some(2),
        samples: None
    };

    let device = audio_subsystem.open_playback(None, &desired_spec, |spec| {
        println!("{:?}", spec);

        let synth_factory = basic_synth::make_basic_synth_factory(spec.freq as f64);
        let melody_event_list = melody.get_event_list(spec.freq as f64, 0);
        //println!("{:?}", melody_event_list);

        SynthPlayer {
            synth: synth_factory.make_synth(),
            event_queue: melody_event_list,
            time: 0,
            current_event: 0,
        }
        
    }).unwrap();

    device.resume();

    let window = video_subsystem.window("Melody", 80, 60)
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    let mut renderer = window.renderer().build().unwrap();

    renderer.set_draw_color(Color::RGB(255, 0, 0));
    renderer.clear();
    renderer.present();

    let mut event_pump = sdl_context.event_pump().unwrap();

    'running: loop {
        for event in event_pump.wait_timeout_iter(10) {
            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                _ => {}
            }
        }
    }
}