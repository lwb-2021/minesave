use std::error::Error;

pub fn report_err<E>(msg: &'static str) -> Box<dyn Fn(&E)>
where
    E: Error,
{
    Box::new(fun(move |e: &E| error!("{}: {:?}", msg, e)))
}

fn fun<T, F: Fn(&T)>(f: F) -> F {
    f
}
