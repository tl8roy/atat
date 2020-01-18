#![no_std]

#[allow(dead_code)]
#[allow(unused_imports)]

mod errors;
mod command_prefix;
mod cursor;

//https://www.reddit.com/r/rust/comments/3tqccq/associated_type_with_lifetime_wont_compile/
pub trait ATATCreate<'a> {
    type ATATState;
    type Buffer;

    fn create_command(&self, buffer_out: Self::Buffer) -> Result<Self::ATATState,ATATError>;
    //where T: core::convert::From<&'a[u8]>;
}
pub trait ATATParse<'a> {
    type ATATState;
    type Buffer;
    fn parse_response(state: Self::ATATState, buffer_in: Self::Buffer) -> Result<Self,ATATError>
    where Self: core::marker::Sized;
}

#[cfg(network)]
pub trait ATATNetwork {
    fn init_network();

}

#[cfg(network)]
pub trait ATATUDP {
    fn send_udp();
    fn recieve_udp();
}

#[cfg(network)]
pub trait ATATTCP {
    fn send_tcp();
    fn recieve_tcp();
}


use embedded_hal::serial;
#[cfg(hal)]
pub trait ATATHAL {
    //fn init(uart: U where Serial );
    fn init_module<U, S, B, E>(serial: S, buffer: B) -> Self
    where
        S: serial::Read<U, Error = E> + serial::Write<U, Error = E>,
        B: Iterator,
        Self: ATATCreate + ATATParse;
    fn get_serial(&self) -> S where S: serial::Read<U, Error = E> + serial::Write<U, Error = E>;
    fn get_buffer(&self) -> B where B: Iterator<Item = u8>;
    fn send_command(&mut self) -> Result<(),ATATError> {
        let b = self.get_buffer();
        let s = self.get_serial();
        for byte in b { 
            s.write(byte).map_error(|e| ATATError::SerialError)?
        }

        s.flush().map_error(|e| ATATError::SerialError)?
    }
    /*
    let telit_module_hal = ATAT::init(UART1,buf);

    let res = telit_module_hal.send_rec(TelitCommand::CGI(arg1:12,arg2:"oeir"));

    let res = telit_module_hal.send_rec_a(TelitCommand::CGI(arg1:12,arg2:"oeir")).async;


    let res = telit_module_hal.send(TelitCommand::CGI(arg1:12,arg2:"oeir"));

    let res = telit_module_hal.receive(TelitCommand::CGI(arg1:12,arg2:"oeir"));
    let (ret1,ret2) = res.unwrap()

    let res = telit_module_hal.send_a(TelitCommand::CGI(arg1:12,arg2:"oeir")).async;

    let res = telit_module_hal.receive_a(TelitCommand::CGI(arg1:12,arg2:"oeir")).async;
    */
}

/*pub trait ATATDNS {
    fn send_dns();
    fn recieve_dns();
}

pub trait ATATHTTP {
    fn send_http();
    fn recieve_http();
}*/


//Export the Errors and macro
pub use errors::ATATError;
pub use command_prefix::*;
pub use atat_derive::atat;
pub use cursor::Cursor;


#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {
        #[allow(dead_code)]
        #[allow(unused_imports)]

        use crate::ATATParse;
        use crate::ATATCreate;

        use atat_derive::atat;
        use crate::errors::ATATError;
        use crate::command_prefix::*;
        use crate::cursor::Cursor;

        struct Testing {
            test: f32
        };

        #[atat("\n")]
        enum TelitCommand {
            AT (None,(u8,u16), ()),
            AT_ (None,(u8,u16), (i64)),
            AT__ (None,(&'b[u8]), (u32)),
            _CGI,
            UBD (u64)
            /*AT {
                sym: "+",
                input: (u8,u16),
                return_val: {test: i32}
            },*/
        }

        //let mut buf: [u8;10] = [0;10];
        let mut cursor = Cursor::new([0; 256]);
        let cursor: &mut Cursor<[u8]> = &mut cursor;

        let t = TelitCommandCreate::AT(1,2);
        //let t = TelitCommand::CGI;
        t.create_command(cursor).expect("ATAT Failed ");
    }
}



/*

let config = PargerConfig {
        concat = true,
        eol = "\n"
    },

atat!("TelitCommand",
    {
        ("+CGI","arg1: f64, arg2: &str","ret1:i32,ret2:[u8]"),
        ("+CGARTI","arg1: f64, arg2: &str","ret1:i32,ret2:[u8]"),

    },config
)-> impl
    cgi_encode() -> Result(T,ParseError)
    cgi_parse() -> Result((),ParseError)

    command_encode(T) -> Result(U,ParseError)
    response_decode() -> Result((),ParseError)




*/