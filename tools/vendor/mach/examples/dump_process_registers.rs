//! A script to read and dump to stdout the current register values of a
//! process.

extern crate libc;
extern crate mach;

use std::io;
use std::mem;
use std::ptr;

use mach::kern_return::KERN_SUCCESS;
use mach::mach_types::{task_t, thread_act_array_t};
use mach::message::mach_msg_type_number_t;
use mach::port::mach_port_name_t;
use mach::structs::x86_thread_state64_t;
use mach::task::{task_resume, task_suspend, task_threads};
use mach::thread_act::thread_get_state;
use mach::thread_status::x86_THREAD_STATE64;
use mach::traps::{mach_task_self, task_for_pid};

use std::io::prelude::*;

fn read_int() -> Result<::libc::c_int, ()> {
    let stdin = io::stdin();
    let mut line = String::new();

    stdin.read_line(&mut line).ok().unwrap();
    let mut value: ::libc::c_int = 0;

    for c in line.chars().take_while(|&c| c != '\n') {
        if let Some(d) = c.to_digit(10) {
            value = value * 10 + (d as ::libc::c_int);
        } else {
            return Err(());
        }
    }
    return Ok(value);
}

fn resume(task: task_t) {
    unsafe {
        let kret = task_resume(task);
        if kret != KERN_SUCCESS {
            println!("Did not succeed in resuming task.");
            println!("kern_return_t error {}", kret);
            panic!();
        }
    }
}

fn main() {
    print!("Enter pid: ");
    io::stdout().flush().ok();

    let pid = match read_int() {
        Ok(v) => v,
        Err(_) => {
            println!("Bad pid!");
            return;
        }
    };

    println!("pid = {}", &pid);

    let task: mach_port_name_t = 0;
    unsafe {
        let kret = task_for_pid(
            mach_task_self() as mach_port_name_t,
            pid,
            mem::transmute(&task),
        );
        if kret != KERN_SUCCESS {
            println!("Did not succeed in getting task for pid {}", pid);
            println!("kern_return_t error {}", kret);
            println!("");
            println!("Did you forget to run with 'sudo'? This script will");
            println!("probably fail without it.");
            return;
        }
    }

    println!("task = 0x{:x}", &task);

    unsafe {
        let kret = task_suspend(task as task_t);
        if kret != KERN_SUCCESS {
            println!("Did not succeed in suspending task.");
            println!("kern_return_t error {}", kret);
            return;
        }
    }

    let thread_list: thread_act_array_t = ptr::null_mut();
    let thread_count: mach_msg_type_number_t = 0;
    unsafe {
        let kret = task_threads(
            task as task_t,
            mem::transmute(&thread_list),
            mem::transmute(&thread_count),
        );
        if kret != KERN_SUCCESS {
            println!("Did not succeed in getting task's threads");
            println!("kern_return_t error {}", kret);
            resume(task as task_t);
            return;
        }
    }

    println!("Task is running {} threads", &thread_count);

    unsafe {
        let threads =
            Vec::from_raw_parts(thread_list, thread_count as usize, thread_count as usize);
        let state = x86_thread_state64_t::new();
        let state_count = x86_thread_state64_t::count();
        for (idx, &thread) in threads.iter().enumerate() {
            println!("Thread {}:", idx);
            let kret = thread_get_state(
                thread,
                x86_THREAD_STATE64,
                mem::transmute(&state),
                mem::transmute(&state_count),
            );
            if kret != KERN_SUCCESS {
                println!("Did not succeed in getting task's thread state");
                println!("kern_return_t error {}", kret);
                continue;
            }

            println!("{:?}", state);
        }
    }

    resume(task as task_t);
    println!("Success!");
}
