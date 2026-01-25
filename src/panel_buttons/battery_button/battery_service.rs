use gtk::glib;
use std::cell::RefCell;
use std::process::Command;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct BatteryMetrics {
  pub percentage: u8,
  pub plugged_in: bool,
  pub estimated_time: Option<String>,
}

type BatteryCallback = Box<dyn Fn(BatteryMetrics)>;

struct BatteryServiceState {
  metrics: BatteryMetrics,
  subscribers: Vec<BatteryCallback>,
  running: bool,
}

thread_local! {
  static BATTERY_SERVICE: RefCell<Option<Rc<RefCell<BatteryServiceState>>>> = RefCell::new(None);
}

pub struct BatteryService;

impl BatteryService {
  pub fn start() -> BatteryMetrics {
    BATTERY_SERVICE.with(|service| {
      if let Some(state) = service.borrow().as_ref() {
        return state.borrow().metrics.clone();
      }

      let metrics = BatteryMetrics {
        percentage: get_battery_percentage(),
        plugged_in: check_if_plugged_in(),
        estimated_time: get_time_till(false),
      };

      let state = Rc::new(RefCell::new(BatteryServiceState {
        metrics: metrics.clone(),
        subscribers: Vec::new(),
        running: true,
      }));

      *service.borrow_mut() = Some(state.clone());

      glib::timeout_add_seconds_local(1, move || {
        let mut s = state.borrow_mut();

        if !s.running {
          return glib::ControlFlow::Break;
        }

        let new_metrics = BatteryMetrics {
          percentage: get_battery_percentage(),
          plugged_in: check_if_plugged_in(),
          estimated_time: get_time_till(s.metrics.plugged_in),
        };

        s.metrics = new_metrics.clone();

        // Notify all subscribers
        for subscriber in &s.subscribers {
          subscriber(new_metrics.clone());
        }

        glib::ControlFlow::Continue
      });

      metrics
    })
  }

  pub fn stop() {
    BATTERY_SERVICE.with(|service| {
      if let Some(state) = service.borrow().as_ref() {
        state.borrow_mut().running = false;
      }
      *service.borrow_mut() = None;
    });
  }

  pub fn subscribe<F>(callback: F)
  where
    F: Fn(BatteryMetrics) + 'static
  {
    BATTERY_SERVICE.with(|service| {
      if let Some(state) = service.borrow().as_ref() {
        state.borrow_mut().subscribers.push(Box::new(callback));
      }
    });
  }

  pub fn _get_current_state() -> Option<BatteryMetrics> {
    BATTERY_SERVICE.with(|service| {
      service.borrow().as_ref().map(|state| {
        state.borrow().metrics.clone()
      })
    })
  }
}

fn get_battery_percentage() -> u8 {
  let output = Command::new("bash")
    .arg("-c")
    .arg("cat /sys/class/power_supply/BAT0/capacity")
    .output()
    .expect("Failed to execute command");

  if output.status.success() {
    let stdout = String::from_utf8_lossy(&output.stdout);
    if let Ok(percentage) = stdout.trim().parse::<u8>() {
      return percentage;
    }
  }

  0
}

fn check_if_plugged_in() -> bool {
  let output = Command::new("bash")
    .arg("-c")
    .arg("cat /sys/class/power_supply/AC/online")
    .output()
    .expect("Failed to execute command");

  if output.status.success() {
    let stdout = String::from_utf8_lossy(&output.stdout);
    return stdout.trim() == "1";
  }

  false
}

fn get_time_till(plugged_in: bool) -> Option<String> {
  let grep_text = if plugged_in {
    "time to full"
  } else {
    "time to empty"
  };

  let postfix = if plugged_in {
    "until full"
  } else {
    "remaining"
  };

  let output = Command::new("bash")
    .arg("-c")
    .arg(format!("upower -i /org/freedesktop/UPower/devices/battery_BAT0 | grep \"{}\" | cut -d: -f2- | xargs", grep_text))
    .output()
    .expect("Failed to execute command");

  if output.status.success() {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let time_str = stdout.trim();
    if !time_str.is_empty() {
      return Some(format!("Estimated {} {}", time_str, postfix));
    }
  }

  None
}