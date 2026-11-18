#[cfg(test)]
mod encoding;

#[cfg(test)]
mod integration;

// chrono reads `TZ` only on Unix
#[cfg(all(test, unix))]
mod local_time;

#[cfg(test)]
mod pointers;
