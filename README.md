
# Sofia-SIP

Rust bindings for Sofia-SIP.


## Usage

Add the following to your `Cargo.toml`:

```toml
[dependencies]
sofia-sip = "0.1.0"
```

Also, make sure you have sofia-sip C library installed in your system (`pkg-config sofia-sip-ua --modversion` is working), in ubuntu and other debian based systems, you can install it by doing:

```bash
sudo apt install libsofia-sip-ua-dev
```

Future versions will have an option (via cargo features) to build and bundle C sofia-sip as a static library.

### Example

```rust
use sofia_sip::{Handle, Nua, NuaEvent, Sip, Tag, TagBuilder};

fn main() {
    /*
    A                    B
    |-------MESSAGE----->|
    |<--------200--------|
    |                    |

    A                    B
    |<------MESSAGE------|
    |--------200-------->|
    |                    |
                           ______(NETWORK)_____
                          /                    \
    A                 NUA STACK (A)
    |                     |
    |    nua::handle( )   |
    |-------------------->|
    |                     |
    |  handle::message()  |
    |------------------->[_]      [MESSAGE]
    |                    [_]------------------>
    |                    [_]
    |                    [_]
    |                    [_]      [200 OK]
    |    ReplyMessage    [_]<------------------
    |<------------------ [_]
    |                     |

                           ______(NETWORK)_____
                          /                    \
    A                 NUA STACK (A)
    |                     |       [MESSAGE]
    |  IncomingMessage   [_]<------------------
    |<-------------------[_]
    |   nua::handle(A')  [_]      [200 OK]
    |                    [_]------------------>
    |                     |
    |                     |
    */

    /* bind on :5080 */
    let sip_bind_url = "sip:*:5080";

    /* send to a SIP contact running in 192.168.0.51 on default port */
    let sip_to_url = "sip:600@192.168.0.51:5060";

    /* build params for Nua::create */
    let tags = TagBuilder::default()
        .tag(Tag::NuUrl(sip_bind_url).unwrap())
        .collect();

    /* create NUA stack */
    let mut nua = Nua::create(&tags).unwrap();

    /*
    Handling of the events coming from NUA stack is done
    in the callback function that is registered for NUA stack
    */
    nua.callback(
        |nua: &mut Nua,
         event: NuaEvent,
         status: u32,
         phrase: String,
         _handle: Option<&Handle>,
         sip: Sip,
         _tags: Vec<Tag>| {
            println!("({:?}) status: {} | {}", &event, status, &phrase);
            match event {
                NuaEvent::ReplyShutdown => { /* received when NUA stack is about to shutdown */ }
                NuaEvent::IncomingMessage => {
                    /* incoming NEW message */
                    println!("Received MESSAGE: {} {}", status, &phrase);
                    println!("From: {}", sip.from());
                    println!("To: {}", sip.to());
                    println!("Subject: {}", sip.subject());
                    println!("ContentType: {}", sip.content_type());
                    println!("Payload: {:?}", sip.payload().as_utf8_lossy());

                    /* quit after new message */
                    nua.quit();
                }
                NuaEvent::ReplyMessage => {
                    /* quit if response != 2XX */
                    if status < 200 || status >= 300 {
                        nua.quit();
                    }
                }
                _ => {}
            }
        },
    );

    /* Message to be send */
    let my_message = "Hi Sofia-SIP-sys";

    /* build params for Handle::create */
    let tags = TagBuilder::default()
        .tag(Tag::SipTo(sip_to_url).unwrap())
        .tag(Tag::NuUrl(sip_to_url).unwrap())
        .collect();

    /* create operation handle */
    let handle = Handle::create(&nua, &tags).unwrap();

    /* build params for handle.message() */
    let tags = TagBuilder::default()
        .tag(Tag::SipSubject("NUA").unwrap())
        .tag(Tag::SipTo(sip_to_url).unwrap())
        .tag(Tag::NuUrl(sip_to_url).unwrap())
        .tag(Tag::SipContentType("text/plain").unwrap())
        .tag(Tag::SipPayloadString(my_message).unwrap())
        .collect();

    /* The message() function enqueue a SIP MESSAGE on NUA STACK */
    handle.message(&tags);

    /* enter main loop for processing of messages */
    println!("enter the main loop");
    nua.run();
    println!("the main loop exit");
}
```

## Documentation

Sofia-SIP Rust bindings tries to mimic almost as possible the API of Sofia-SIP C library. You can start by learning the concepts of [Sofia SIP User Agent Library - "nua" - High-Level User Agent Module](http://sofia-sip.sourceforge.net/refdocs/nua/).

After this intro, please read [examples](https://github.com/iuridiniz/sofia-sip-sys/blob/main/examples) or [the tests from tests directory](https://github.com/iuridiniz/sofia-sip-sys/blob/main/tests).

### Sofia-SIP C docs
#### NUA documentation

The best way to learn sofia-sip is by learning how to use the NUA engine, their call model and how NUA events are presented and handled.

- ["NUA" - High-Level User Agent Module](http://sofia-sip.sourceforge.net/refdocs/nua/).
  - The NUA engine hides many low-level signaling and media management aspects from the application programmer.
- [NUA Call Model](http://sofia-sip.sourceforge.net/refdocs/nua/nua_call_model.html).
  - The call model is used to present changes in call: when media starts to flow, when call is considered established, when call is terminated.
- [NUA Event Diagrams](http://sofia-sip.sourceforge.net/refdocs/nua/nua_event_diagrams.html)
  - Example diagrams to present how to use NUA API with different SIP use cases.

#### SOFIA Modules

[Sofia SIP User Agent Library - sofia-sip-ua](http://sofia-sip.sourceforge.net/refdocs/index.html)

Common runtime library:
- [Sofia SIP User Agent Library - "su" - OS Services and Utilities](http://sofia-sip.sourceforge.net/refdocs/su/index.html).
- [Sofia SIP User Agent Library - "sresolv" - Asynchronous DNS Resolver](http://sofia-sip.sourceforge.net/refdocs/sresolv/index.html).
- [Sofia SIP User Agent Library - "ipt" - Utility Module](http://sofia-sip.sourceforge.net/refdocs/ipt/index.html).

SIP Signaling:
- [Sofia SIP User Agent Library - "nua" - High-Level User Agent Module](http://sofia-sip.sourceforge.net/refdocs/nua/).
- [Sofia SIP User Agent Library - "nea" - SIP Events Module](http://sofia-sip.sourceforge.net/refdocs/nea/index.html).
- [Sofia SIP User Agent Library - "iptsec" - Authentication Module](http://sofia-sip.sourceforge.net/refdocs/iptsec/index.html).
- [Sofia SIP User Agent Library - "nta" - SIP Transactions Module](http://sofia-sip.sourceforge.net/refdocs/nta/index.html).
- [Sofia SIP User Agent Library - "tport" - Transport Module](http://sofia-sip.sourceforge.net/refdocs/tport/index.html).
- [Sofia SIP User Agent Library - "sip" - SIP Parser Module](http://sofia-sip.sourceforge.net/refdocs/sip/index.html).
- [Sofia SIP User Agent Library - "msg" - Message Parser Module](http://sofia-sip.sourceforge.net/refdocs/msg/index.html).
- [Sofia SIP User Agent Library - "url" - URL Module](http://sofia-sip.sourceforge.net/refdocs/url/index.html).
- [Sofia SIP User Agent Library - "bnf" - String Parser Module](http://sofia-sip.sourceforge.net/refdocs/bnf/index.html).

HTTP subsystem:
- [Sofia SIP User Agent Library - "nth" - HTTP Transactions Module](http://sofia-sip.sourceforge.net/refdocs/nth/index.html).
- [Sofia SIP User Agent Library - "http" - HTTP Parser Module](http://sofia-sip.sourceforge.net/refdocs/http/index.html).

SDP processing:
- [Sofia SIP User Agent Library - "soa" - SDP Offer/Answer Engine Module](http://sofia-sip.sourceforge.net/refdocs/soa/index.html).
- [Sofia SIP User Agent Library - "sdp" - SDP Module](http://sofia-sip.sourceforge.net/refdocs/sdp/index.html).

Other:
- [Sofia SIP User Agent Library - "features" Module](http://sofia-sip.sourceforge.net/refdocs/features/index.html).
- [Sofia SIP User Agent Library - "stun" - STUN Client and Server Module](http://sofia-sip.sourceforge.net/refdocs/stun/index.html).

## Acknowledgements

 - [Original Sofia-SIP (not currently maintained)](https://sourceforge.net/projects/sofia-sip/).
 - [Freeswitch version of Sofia-SIP (Currently maintained)](https://github.com/freeswitch/sofia-sip/).


## Authors

- [@iuridiniz](https://www.github.com/iuridiniz)


## License

- Rust bindings: [MIT](https://choosealicense.com/licenses/mit/)
- Sofia-SIP C library: [LGPL-2.1-or-later](https://choosealicense.com/licenses/lgpl-2.1/)

Before compiling statically, please read [this](https://www.gnu.org/licenses/gpl-faq.html#LGPLStaticVsDynamic).

## Roadmap
- Version 0.1.0 -> DONE
    - NUA: Basic support to send and receive SIP MESSAGE's, allowing to create a chat using SIP.

- Version 0.2.0
    - NUA: Basic support to send SIP INVITE(SDP)/REGISTER(auth) and receive SIP INVITE(SDP), allowing to create a simple soft phone.
    - Others modules: basic support to make NUA objectives work.

- Version 0.3.0
    - NUA: Support receive SIP REGISTER(auth), allowing to create a simple SIP PBX.
    - Others modules: basic support to make NUA objectives work.

- Version 1.0.0
    - NUA: Full bindings for NUA.
    - SDP: Full support for SDP parsing.

NUA is the High-Level User Agent Module of lib-sofia. To learn more about sofia modules, go to [reference documentation for libsofia-sip-ua submodules](http://sofia-sip.sourceforge.net/refdocs/building.html).


