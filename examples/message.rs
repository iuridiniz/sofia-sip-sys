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
