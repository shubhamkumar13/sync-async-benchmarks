#![feature(trace_macros)]
#![feature(box_syntax)]
#![feature(log_syntax)]

use hyper::{
  service::{make_service_fn, service_fn},
  Body, Request, Response, Server,
};
use std::{convert::Infallible, net::SocketAddr};

use execution_engine::{self, eval, expr::*, ivec, runtime};

async fn program() -> Expr {
  elet("range",
       esfn("Int", "range", 0, ivec![eint(1), eint(100),]),
       esfn("List",
            "map",
            0,
            ivec![(evar("range")),
                  elambda(ivec!["i"],
                          eif(ebinop(ebinop(evar("i"),
                                            "Int",
                                            "%",
                                            0,
                                            eint(15)),
                                     "Int",
                                     "==",
                                     0,
                                     eint(0)),
                              estr("fizzbuzz"),
                              eif(ebinop(ebinop(evar("i"),
                                                "Int",
                                                "%",
                                                0,
                                                eint(5)),
                                         "Int",
                                         "==",
                                         0,
                                         eint(0)),
                                  estr("buzz"),
                                  eif(ebinop(ebinop(evar("i"),
                                                    "Int",
                                                    "%",
                                                    0,
                                                    eint(3)),
                                             "Int",
                                             "==",
                                             0,
                                             eint(0)),
                                      estr("fizz"),
                                      esfn("Int",
                                           "toString",
                                           0,
                                           ivec![evar("i")])))))]))
}

async fn run_program(_req: Request<Body>)
                     -> Result<Response<Body>, Infallible> {
  let tlid = runtime::TLID::TLID(7);
  let state =
    eval::ExecState { caller: runtime::Caller::Toplevel(tlid), };
  let result;
  {
    let prog = program().await;
    result = eval::run_json(&state, prog);
  }
  Ok(Response::new(result.to_string().into()))
}

#[tokio::main]
async fn main() {
  let port = std::fs::read_to_string("port").unwrap()
                                            .trim()
                                            .parse::<u16>()
                                            .unwrap();
  let addr = SocketAddr::from(([127, 0, 0, 1], port));
  // A `Service` is needed for every connection, so this
  // creates one from our `hello_world` function.
  let make_svc = make_service_fn(|_conn| async {
    // service_fn converts our function into a `Service`
    Ok::<_, Infallible>(service_fn(run_program))
  });

  let server = Server::bind(&addr).serve(make_svc);

  // Run this server for... forever!
  if let Err(e) = server.await {
    eprintln!("server error: {}", e);
  }
}
