

pub fn try_use<I, F, R, E>(iter: I, mut f: F) -> Result<R, Vec<E>>
where I: IntoIterator,
      F: FnMut(<I as IntoIterator>::Item) -> Result<R, E>
{
    let mut errors = Vec::new();

    for i in iter {
        match f(i) {
            Ok(r) => return Ok(r),
            Err(e) => errors.push(e),
        }
    }
    Err(errors)
}
