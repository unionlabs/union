use crate::runtime::blocking::NoopSchedule;
use crate::runtime::task::Inject;
use crate::runtime::thread_pool::queue;
use crate::runtime::MetricsBatch;

use loom::thread;

#[test]
fn basic() {
    loom::model(|| {
        let (steal, mut local) = queue::local();
        let inject = Inject::new();
        let mut metrics = MetricsBatch::new();

        let th = thread::spawn(move || {
            let mut metrics = MetricsBatch::new();
            let (_, mut local) = queue::local();
            let mut n = 0;

            for _ in 0..3 {
                if steal.steal_into(&mut local, &mut metrics).is_some() {
                    n += 1;
                }

                while local.pop().is_some() {
                    n += 1;
                }
            }

            n
        });

        let mut n = 0;

        for _ in 0..2 {
            for _ in 0..2 {
                let (task, _) = super::unowned(async {});
                local.push_back(task, &inject, &mut metrics);
            }

            if local.pop().is_some() {
                n += 1;
            }

            // Push another task
            let (task, _) = super::unowned(async {});
            local.push_back(task, &inject, &mut metrics);

            while local.pop().is_some() {
                n += 1;
            }
        }

        while inject.pop().is_some() {
            n += 1;
        }

        n += th.join().unwrap();

        assert_eq!(6, n);
    });
}

#[test]
fn steal_overflow() {
    loom::model(|| {
        let (steal, mut local) = queue::local();
        let inject = Inject::new();
        let mut metrics = MetricsBatch::new();

        let th = thread::spawn(move || {
            let mut metrics = MetricsBatch::new();
            let (_, mut local) = queue::local();
            let mut n = 0;

            if steal.steal_into(&mut local, &mut metrics).is_some() {
                n += 1;
            }

            while local.pop().is_some() {
                n += 1;
            }

            n
        });

        let mut n = 0;

        // push a task, pop a task
        let (task, _) = super::unowned(async {});
        local.push_back(task, &inject, &mut metrics);

        if local.pop().is_some() {
            n += 1;
        }

        for _ in 0..6 {
            let (task, _) = super::unowned(async {});
            local.push_back(task, &inject, &mut metrics);
        }

        n += th.join().unwrap();

        while local.pop().is_some() {
            n += 1;
        }

        while inject.pop().is_some() {
            n += 1;
        }

        assert_eq!(7, n);
    });
}

#[test]
fn multi_stealer() {
    const NUM_TASKS: usize = 5;

    fn steal_tasks(steal: queue::Steal<NoopSchedule>) -> usize {
        let mut metrics = MetricsBatch::new();
        let (_, mut local) = queue::local();

        if steal.steal_into(&mut local, &mut metrics).is_none() {
            return 0;
        }

        let mut n = 1;

        while local.pop().is_some() {
            n += 1;
        }

        n
    }

    loom::model(|| {
        let (steal, mut local) = queue::local();
        let inject = Inject::new();
        let mut metrics = MetricsBatch::new();

        // Push work
        for _ in 0..NUM_TASKS {
            let (task, _) = super::unowned(async {});
            local.push_back(task, &inject, &mut metrics);
        }

        let th1 = {
            let steal = steal.clone();
            thread::spawn(move || steal_tasks(steal))
        };

        let th2 = thread::spawn(move || steal_tasks(steal));

        let mut n = 0;

        while local.pop().is_some() {
            n += 1;
        }

        while inject.pop().is_some() {
            n += 1;
        }

        n += th1.join().unwrap();
        n += th2.join().unwrap();

        assert_eq!(n, NUM_TASKS);
    });
}

#[test]
fn chained_steal() {
    loom::model(|| {
        let mut metrics = MetricsBatch::new();
        let (s1, mut l1) = queue::local();
        let (s2, mut l2) = queue::local();
        let inject = Inject::new();

        // Load up some tasks
        for _ in 0..4 {
            let (task, _) = super::unowned(async {});
            l1.push_back(task, &inject, &mut metrics);

            let (task, _) = super::unowned(async {});
            l2.push_back(task, &inject, &mut metrics);
        }

        // Spawn a task to steal from **our** queue
        let th = thread::spawn(move || {
            let mut metrics = MetricsBatch::new();
            let (_, mut local) = queue::local();
            s1.steal_into(&mut local, &mut metrics);

            while local.pop().is_some() {}
        });

        // Drain our tasks, then attempt to steal
        while l1.pop().is_some() {}

        s2.steal_into(&mut l1, &mut metrics);

        th.join().unwrap();

        while l1.pop().is_some() {}
        while l2.pop().is_some() {}
        while inject.pop().is_some() {}
    });
}
