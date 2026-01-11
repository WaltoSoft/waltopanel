use gtk::glib;
use std::cell::RefCell;
use std::fs;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct CpuMetrics {
  pub overall_usage: f32,
  pub per_core_usage: Vec<f32>,
}

#[derive(Debug, Clone)]
pub struct MemoryMetrics {
  pub usage_percentage: f32,
  pub used_mb: u64,
  pub total_mb: u64,
}

#[derive(Debug, Clone)]
pub struct SystemMetrics {
  pub cpu: CpuMetrics,
  pub memory: MemoryMetrics,
}

type MetricsCallback = Box<dyn Fn(SystemMetrics)>;

struct SystemMetricsServiceState {
  metrics: SystemMetrics,
  subscribers: Vec<MetricsCallback>,
  running: bool,
  prev_cpu_stats: Option<CpuStats>,
}

#[derive(Debug, Clone)]
struct CpuStats {
  overall: CoreStats,
  per_core: Vec<CoreStats>,
}

#[derive(Debug, Clone)]
struct CoreStats {
  idle: u64,
  total: u64,
}

thread_local! {
  static METRICS_SERVICE: RefCell<Option<Rc<RefCell<SystemMetricsServiceState>>>> = RefCell::new(None);
}

pub struct SystemMetricsService;

impl SystemMetricsService {
  pub fn start() -> SystemMetrics {
    METRICS_SERVICE.with(|service| {
      if let Some(state) = service.borrow().as_ref() {
        return state.borrow().metrics.clone();
      }

      // Read initial CPU stats for next calculation
      let initial_cpu_stats = read_cpu_stats();

      let metrics = SystemMetrics {
        cpu: CpuMetrics {
          overall_usage: 0.0,
          per_core_usage: vec![0.0; initial_cpu_stats.per_core.len()],
        },
        memory: get_memory_metrics(),
      };

      let state = Rc::new(RefCell::new(SystemMetricsServiceState {
        metrics: metrics.clone(),
        subscribers: Vec::new(),
        running: true,
        prev_cpu_stats: Some(initial_cpu_stats),
      }));

      *service.borrow_mut() = Some(state.clone());

      glib::timeout_add_seconds_local(1, move || {
        let mut s = state.borrow_mut();

        if !s.running {
          return glib::ControlFlow::Break;
        }

        // Read current CPU stats
        let current_cpu_stats = read_cpu_stats();

        // Calculate CPU usage based on delta
        let cpu_metrics = if let Some(ref prev_stats) = s.prev_cpu_stats {
          calculate_cpu_usage(prev_stats, &current_cpu_stats)
        } else {
          CpuMetrics {
            overall_usage: 0.0,
            per_core_usage: vec![0.0; current_cpu_stats.per_core.len()],
          }
        };

        let new_metrics = SystemMetrics {
          cpu: cpu_metrics,
          memory: get_memory_metrics(),
        };

        s.metrics = new_metrics.clone();
        s.prev_cpu_stats = Some(current_cpu_stats);

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
    METRICS_SERVICE.with(|service| {
      if let Some(state) = service.borrow().as_ref() {
        state.borrow_mut().running = false;
      }
      *service.borrow_mut() = None;
    });
  }

  pub fn subscribe<F>(callback: F)
  where
    F: Fn(SystemMetrics) + 'static
  {
    METRICS_SERVICE.with(|service| {
      if let Some(state) = service.borrow().as_ref() {
        state.borrow_mut().subscribers.push(Box::new(callback));
      }
    });
  }

  pub fn get_current_state() -> Option<SystemMetrics> {
    METRICS_SERVICE.with(|service| {
      service.borrow().as_ref().map(|state| {
        state.borrow().metrics.clone()
      })
    })
  }
}

fn read_cpu_stats() -> CpuStats {
  let content = fs::read_to_string("/proc/stat")
    .unwrap_or_default();

  let mut overall = CoreStats { idle: 0, total: 0 };
  let mut per_core = Vec::new();

  for line in content.lines() {
    if line.starts_with("cpu ") {
      overall = parse_cpu_line(line);
    } else if line.starts_with("cpu") {
      per_core.push(parse_cpu_line(line));
    }
  }

  CpuStats { overall, per_core }
}

fn parse_cpu_line(line: &str) -> CoreStats {
  let parts: Vec<&str> = line.split_whitespace().collect();

  if parts.len() < 5 {
    return CoreStats { idle: 0, total: 0 };
  }

  // CPU line format: cpu user nice system idle iowait irq softirq steal guest guest_nice
  let values: Vec<u64> = parts[1..].iter()
    .filter_map(|s| s.parse::<u64>().ok())
    .collect();

  let idle = values.get(3).copied().unwrap_or(0);
  let total: u64 = values.iter().sum();

  CoreStats { idle, total }
}

fn calculate_cpu_usage(prev: &CpuStats, current: &CpuStats) -> CpuMetrics {
  let overall_usage = calculate_core_usage(&prev.overall, &current.overall);

  let per_core_usage: Vec<f32> = prev.per_core.iter()
    .zip(current.per_core.iter())
    .map(|(prev_core, curr_core)| calculate_core_usage(prev_core, curr_core))
    .collect();

  CpuMetrics {
    overall_usage,
    per_core_usage,
  }
}

fn calculate_core_usage(prev: &CoreStats, current: &CoreStats) -> f32 {
  let total_delta = current.total.saturating_sub(prev.total);
  let idle_delta = current.idle.saturating_sub(prev.idle);

  if total_delta == 0 {
    return 0.0;
  }

  let usage_delta = total_delta.saturating_sub(idle_delta);
  (usage_delta as f32 / total_delta as f32) * 100.0
}

fn get_memory_metrics() -> MemoryMetrics {
  let content = fs::read_to_string("/proc/meminfo")
    .unwrap_or_default();

  let mut total_kb = 0u64;
  let mut available_kb = 0u64;

  for line in content.lines() {
    if line.starts_with("MemTotal:") {
      total_kb = line.split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    } else if line.starts_with("MemAvailable:") {
      available_kb = line.split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    }
  }

  let used_kb = total_kb.saturating_sub(available_kb);
  let usage_percentage = if total_kb > 0 {
    (used_kb as f32 / total_kb as f32) * 100.0
  } else {
    0.0
  };

  MemoryMetrics {
    usage_percentage,
    used_mb: used_kb / 1024,
    total_mb: total_kb / 1024,
  }
}
