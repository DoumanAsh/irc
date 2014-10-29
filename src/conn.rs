use std::cell::{RefCell, RefMut};
use std::io::{BufferedReader, BufferedWriter, IoResult, TcpStream, Writer};
use data::{IrcReader, IrcWriter, Message};

pub struct Connection<T, U> where T: IrcWriter, U: IrcReader {
    writer: RefCell<T>,
    reader: RefCell<U>,
}

impl Connection<BufferedWriter<TcpStream>, BufferedReader<TcpStream>> {
    pub fn connect(host: &str, port: u16) -> IoResult<Connection<BufferedWriter<TcpStream>, BufferedReader<TcpStream>>> {
        let socket = try!(TcpStream::connect(host, port));
        Connection::new(BufferedWriter::new(socket.clone()), BufferedReader::new(socket.clone()))
    }
}

impl<T, U> Connection<T, U> where T: IrcWriter, U: IrcReader {
    pub fn new(writer: T, reader: U) -> IoResult<Connection<T, U>> {
        Ok(Connection {
            writer: RefCell::new(writer),
            reader: RefCell::new(reader),
        })
    }

    fn send_internal(&self, msg: &str) -> IoResult<()> {
        let mut send = self.writer.borrow_mut();
        try!(send.write_str(msg));
        send.flush()
    }

    pub fn send(&self, msg: Message) -> IoResult<()> {
        let mut send = msg.command.to_string();
        if msg.args.init().len() > 0 {
            send.push_str(" ");
            send.push_str(msg.args.init().connect(" ")[]);
        }
        send.push_str(" ");
        if msg.colon_flag { send.push_str(":") }
        send.push_str(*msg.args.last().unwrap());
        send.push_str("\r\n");
        self.send_internal(send[])
    }

    pub fn writer<'a>(&'a self) -> RefMut<'a, T> {
        self.writer.borrow_mut()
    }

    pub fn reader<'a>(&'a self) -> RefMut<'a, U> {
        self.reader.borrow_mut()
    }
}

#[cfg(test)]
mod test {
    use super::Connection;
    use std::io::MemWriter;
    use std::io::util::NullReader;
    use data::{IrcReader, Message};

    fn data<U>(conn: Connection<MemWriter, U>) -> String where U: IrcReader {
        String::from_utf8(conn.writer().deref_mut().get_ref().to_vec()).unwrap()
    }

    #[test]
    fn new_connection() {
        assert!(Connection::new(MemWriter::new(), NullReader).is_ok());
    }

    #[test]
    fn send_internal() {
        let c = Connection::new(MemWriter::new(), NullReader).unwrap();
        c.send_internal("string of text").unwrap();
        assert_eq!(data(c), format!("string of text"));
    }

    #[test]
    fn send() {
        let c = Connection::new(MemWriter::new(), NullReader).unwrap();
        let args = ["flare.to.ca.fyrechat.net"];
        c.send(Message::new(None, "PING", args, true)).unwrap();
        assert_eq!(data(c), format!("PING :flare.to.ca.fyrechat.net\r\n"));
    }
}
