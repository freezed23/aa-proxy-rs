//! Bounded, observational AA NavigationStatus fragment collector.
use std::time::{Duration, Instant};
const LIMIT: usize = 64 * 1024;
#[derive(Default)]
pub(crate) struct NavCapture { pending: Option<(u8, usize, Vec<u8>, Instant, usize)> }
impl NavCapture {
    pub(crate) fn push(&mut self, channel: u8, flags: u8, total: Option<u32>, bytes: &[u8]) -> Option<Vec<u8>> {
        if self.pending.as_ref().is_some_and(|p| p.3.elapsed() > Duration::from_secs(3)) { self.pending = None; }
        match flags & 3 {
            3 => { self.pending = None; if bytes.len() <= LIMIT { Some(bytes.to_vec()) } else { None } },
            1 => {
                self.pending = None;
                let n = total? as usize;
                if bytes.len() < 2 || !matches!(&bytes[..2], [0x80, 6] | [0x80, 7]) || n > LIMIT || n <= bytes.len() { return None; }
                self.pending = Some((channel, n, bytes.to_vec(), Instant::now(), 1)); None
            },
            _ => {
                let mut p = self.pending.take()?;
                if p.0 != channel || p.2.len() + bytes.len() > p.1 || p.4 >= 128 { return None; }
                p.2.extend_from_slice(bytes); p.4 += 1;
                if flags & 3 == 2 { if p.2.len() == p.1 { Some(p.2) } else { None } }
                else { self.pending = Some(p); None }
            }
        }
    }
}
#[cfg(test)] mod tests {
 use super::*;
 #[test] fn single_and_fragments() { let mut c=NavCapture::default(); assert_eq!(c.push(9,3,None,&[0x80,6]),Some(vec![0x80,6])); assert!(c.push(9,1,Some(4),&[0x80,7]).is_none()); assert!(c.push(9,0,None,&[1]).is_none()); assert_eq!(c.push(9,2,None,&[2]),Some(vec![0x80,7,1,2])); }
 #[test] fn rejects_bad_lengths_and_channels() { let mut c=NavCapture::default(); assert!(c.push(9,1,Some(65537),&[0x80,6]).is_none()); c.push(9,1,Some(4),&[0x80,6]); assert!(c.push(8,2,None,&[1,2]).is_none()); assert!(c.push(9,2,None,&[1,2]).is_none()); }
 #[test] fn stale_and_new_first_reset() { let mut c=NavCapture::default(); c.push(9,1,Some(4),&[0x80,6]); c.pending.as_mut().unwrap().3=Instant::now()-Duration::from_secs(4); assert!(c.push(9,2,None,&[1,2]).is_none()); c.push(9,1,Some(4),&[0x80,6]); c.push(9,1,Some(4),&[0x80,7]); assert_eq!(c.push(9,2,None,&[1,2]),Some(vec![0x80,7,1,2])); }
}
