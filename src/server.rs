use hyper::server::conn::AddrIncoming;
use tokio::net::TcpSocket;

pub fn builder() -> hyper::server::Builder<AddrIncoming> {
    let addr = "0.0.0.0:3000".parse().unwrap();
    let socket = TcpSocket::new_v4().unwrap();
    #[cfg(unix)]
    {
        if let Err(e) = socket.set_reuseport(true) {
            eprintln!("无法使用 SO_REUSEPORT: {}", e);
        }
    }
    socket.set_reuseaddr(true).unwrap();
    socket.bind(addr).unwrap();
    let listener = socket.listen(102400).unwrap();
    let incoming = AddrIncoming::from_listener(listener).unwrap();
    axum::Server::builder(incoming)
        .http1_only(true)
        .tcp_nodelay(true)
}
