pub fn report_err<E>(msg: &'static str) -> Box<dyn Fn(&E)>
where
    E: std::error::Error,
{
    Box::new(fun(move |e: &E| error!("{}: {:?}", msg, e)))
}

fn fun<T, F: Fn(&T)>(f: F) -> F {
    f
}
