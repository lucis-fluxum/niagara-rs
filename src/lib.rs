//#[cfg(test)]
//mod tests {
//    use std::net::SocketAddr;
//    use std::str::FromStr;
//
//    use tokio::net::UdpSocket;
//
//    use super::*;
//
//    #[tokio::test]
//    async fn test_sockets() {
//        let socket = UdpSocket::bind("127.0.0.1:3000").await.unwrap();
//        let (mut receiver, mut sender) = socket.split();
//        sender
//            .send_to(b"12345", &SocketAddr::from_str("127.0.0.1:3000").unwrap())
//            .await
//            .unwrap();
//        let mut buf: [u8; 5] = [0, 0, 0, 0, 0];
//        dbg!(receiver.recv_from(&mut buf).await.unwrap());
//        dbg!(String::from_utf8_lossy(&buf));
//    }
//}
