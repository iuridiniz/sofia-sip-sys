use crate::Handle;
use crate::Nua;
use crate::NuaEvent;
use crate::Root;
use crate::Sip;
use crate::Tag;
use crate::TagBuilder;

use crate::su::wrap;
use adorn::adorn;
use serial_test::serial;

use std::cell::RefCell;
use std::rc::Rc;

#[test]
#[adorn(wrap)]
#[serial]
fn create_nua_with_default_root() {
    let tags = TagBuilder::default().collect();

    Nua::create(tags).unwrap();
}

#[test]
#[adorn(wrap)]
#[serial]
fn create_nua_with_custom_root() {
    let tags = TagBuilder::default().collect();
    let root = Root::new().unwrap();

    Nua::create_with_root(&root, tags).unwrap();
}

#[test]
#[adorn(wrap)]
#[serial]
fn nua_set_callback_to_closure() {
    let tags = TagBuilder::default().collect();
    let mut nua = Nua::create(tags).unwrap();
    nua.callback(
        |nua: &mut Nua,
         event: NuaEvent,
         status: u32,
         phrase: String,
         handle: Option<&Handle>,
         sip: Sip,
         tags: Vec<Tag>| {
            dbg!(&nua, &event, &status, &phrase, &handle, &sip, &tags);
        },
    )
}

#[test]
#[adorn(wrap)]
#[serial]
fn nua_set_callback_to_fn() {
    fn cb(
        nua: &mut Nua,
        event: NuaEvent,
        status: u32,
        phrase: String,
        handle: Option<&Handle>,
        sip: Sip,
        tags: Vec<Tag>,
    ) {
        dbg!(&nua, &event, &status, &phrase, &handle, &sip, &tags);
    }

    let tags = TagBuilder::default().collect();
    let mut nua = Nua::create(tags).unwrap();
    nua.callback(cb);
}

#[test]
#[adorn(wrap)]
#[serial]
fn create_nua_full() {
    fn cb(
        nua: &mut Nua,
        event: NuaEvent,
        status: u32,
        phrase: String,
        handle: Option<&Handle>,
        sip: Sip,
        tags: Vec<Tag>,
    ) {
        dbg!(&nua, &event, &status, &phrase, &handle, &sip, &tags);
    }

    let tags = TagBuilder::default().collect();
    let root = Root::new().unwrap();

    let mut nua = Nua::create_full(&root, cb, tags).unwrap();
}

#[test]
#[adorn(wrap)]
#[serial]
fn create_nua_with_custom_url() {
    let url = Tag::NuUrl("sip:*:5080").unwrap();

    let root = Root::new().unwrap();

    let tags = TagBuilder::default().tag(url).collect();

    Nua::create(tags).unwrap();
}

#[test]
#[adorn(wrap)]
#[serial]
fn create_two_nua_with_same_port() {
    let url = Tag::NuUrl("sip:*:5080").unwrap();

    let root = Root::new().unwrap();

    let b = TagBuilder::default();
    let b = b.tag(url);
    let tags = b.collect();

    let _nua_a = Nua::create_with_root(&root, tags).unwrap();

    let url = Tag::NuUrl("sip:*:5080").unwrap();

    let root = Root::new().unwrap();

    let b = TagBuilder::default();
    let b = b.tag(url);
    let tags = b.collect();

    assert!(Nua::create_with_root(&root, tags).is_err());
}

#[test]
#[adorn(wrap)]
#[serial]
fn nua_send_message_to_itself() {
    /* see <lib-sofia-ua-c>/tests/test_simple.c::test_message */
    /*

    A
    |-------------------\
    |<------MESSAGE-----/
    |
    |-------------------\
    |<--------200-------/
    |

                           _(LOOPBACK NETWORK)_
                          /                    \
    A                 NUA STACK (A)
    |                     |
    |   nua::handle(A')   |
    |-------------------->|
    |                     |
    |  handle::message()  |
    |------------------->[_]     [MESSAGE]
    |                    [_]-------------------\
    |                    [_]                   |
    |                    [_]                   |
    |  IncomingMessage   [_]<------------------/
    |<-------------------[_]
    |   nua::handle(A")  [_]
    |                    [_]      [200 OK]
    |                    [_]-------------------\
    |                    [_]                   |
    |    ReplyMessage    [_]<------------------/
    |<------------------ [_]
    |                    [_]
    | handle::destroy()  [_]
    |------------------->[_]
    |                     |
    */
    let my_message = "Hi\n";
    let root = Root::new().unwrap();
    let url = Rc::new("sip:127.0.0.1:9997");

    let mut nua = {
        let url = Tag::NuUrl(&url.clone()).unwrap();
        let tags = TagBuilder::default().tag(url).collect();
        Nua::create_with_root(&root, tags).unwrap()
    };

    let recv_message = Rc::new(RefCell::new(String::new()));

    {
        let recv_message = recv_message.clone();
        nua.callback(
            move |nua: &mut Nua,
                  event: NuaEvent,
                  status: u32,
                  phrase: String,
                  handle: Option<&Handle>,
                  sip: Sip,
                  tags: Vec<Tag>| {
                dbg!(&nua, &event, &status, &phrase, &handle, &sip, &tags);
                let root: &Root = nua.root();
                match event {
                    NuaEvent::ReplyShutdown => {
                        root.break_();
                    }
                    NuaEvent::IncomingMessage => {
                        println!("Received MESSAGE: {} {}", status, &phrase);
                        println!("From: {}", sip.from());
                        println!("To: {}", sip.to());
                        println!("Subject: {}", sip.subject());
                        println!("ContentType: {}", sip.content_type());
                        let payload = sip.payload().as_utf8_lossy();
                        println!("Payload: {:?}", &payload);
                        recv_message.borrow_mut().push_str(&payload);
                    }
                    NuaEvent::ReplyMessage => {}
                    _ => {}
                }
            },
        );
    }

    let handle = {
        let tags = TagBuilder::default()
            .tag(Tag::SipTo(&url.clone()).unwrap())
            .tag(Tag::NuUrl(&url.clone()).unwrap())
            .collect();
        Handle::create(&nua, tags).unwrap()
    };

    let tags = TagBuilder::default()
        .tag(Tag::SipSubject("NUA").unwrap())
        .tag(Tag::SipTo(&url.clone()).unwrap())
        .tag(Tag::NuUrl(&url.clone()).unwrap())
        .tag(Tag::SipContentType("text/plain").unwrap())
        .tag(Tag::SipPayloadString(my_message).unwrap())
        .collect();

    handle.message(tags);
    root.sleep(0);

    assert_eq!(&*recv_message.borrow(), my_message);
}

#[test]
#[adorn(wrap)]
#[serial]
fn test_nua_a_send_message_to_nua_b() {
    /* see <lib-sofia-ua-c>/tests/test_simple.c::test_message */
    /*
    A                    B
    |-------MESSAGE----->|
    |<--------200--------| (method allowed, responded)
    |                    |

                           ______(NETWORK)_____
                          /                    \
    A                 NUA STACK (A)         NUA STACK (B)             B
    |                     |                     |                     |
    |    nua::handle(B)   |                     |                     |
    |-------------------->|                     |                     |
    |                     |                     |                     |
    |  handle::message()  |                     |                     |
    |------------------->[_]      [MESSAGE]     |                     |
    |                    [_]------------------>[_]   IncomingMessage  |
    |                    [_]                   [_]------------------->|
    |                    [_]                   [_]   nua::handle(A)   |
    |                    [_]      [200 OK]     [_]                    |
    |    ReplyMessage    [_]<------------------[_]                    |
    |<------------------ [_]                    |                     |
    |                     |                     |                     |
    |                     |                     |                     |

    */
    let nua_a_url = "sip:127.0.0.1:5080";
    let mut nua_a = {
        let url = Tag::NuUrl(nua_a_url).unwrap();
        let tags = TagBuilder::default().tag(url).collect();
        Nua::create(tags).unwrap()
    };
    let nua_b_url = "sip:127.0.0.1:5081";
    let mut nua_b = {
        let url = Tag::NuUrl(nua_b_url).unwrap();
        let tags = TagBuilder::default().tag(url).collect();
        Nua::create(tags).unwrap()
    };
    let recv_message = Rc::new(RefCell::new(String::new()));
    {
        /* NUA B */
        let recv_message = recv_message.clone();
        nua_b.callback(
            move |nua: &mut Nua,
                  event: NuaEvent,
                  status: u32,
                  phrase: String,
                  handle: Option<&Handle>,
                  sip: Sip,
                  tags: Vec<Tag>| {
                // dbg!(&nua, &event, &status, &phrase, &handle, &sip, &tags);
                let root: &Root = nua.root();
                println!("[NUA _B]Event: {:?}", &event);
                match event {
                    NuaEvent::ReplyShutdown => {}
                    NuaEvent::IncomingMessage => {
                        println!("[NUA _B]Received MESSAGE: {} {}", status, &phrase);
                        println!("[NUA _B]From: {}", sip.from());
                        println!("[NUA _B]To: {}", sip.to());
                        println!("[NUA _B]Subject: {}", sip.subject());
                        println!("[NUA _B]ContentType: {}", sip.content_type());
                        let payload = sip.payload().as_utf8_lossy();
                        println!("[NUA _B]Payload: {:?}", &payload);
                        recv_message.borrow_mut().push_str(&payload);
                    }
                    NuaEvent::ReplyMessage => {}
                    _ => {}
                }
            },
        );
    }
    {
        /* NUA A */
        let recv_message = recv_message.clone();
        nua_a.callback(
            move |nua: &mut Nua,
                  event: NuaEvent,
                  status: u32,
                  phrase: String,
                  handle: Option<&Handle>,
                  sip: Sip,
                  tags: Vec<Tag>| {
                // dbg!(&nua, &event, &status, &phrase, &handle, &sip, &tags);
                let root: &Root = nua.root();
                println!("[NUA A_]Event: {:?}", &event);
                match event {
                    NuaEvent::ReplyShutdown => {}
                    NuaEvent::IncomingMessage => {}
                    NuaEvent::ReplyMessage => {
                        root.break_();
                    }
                    _ => {}
                }
            },
        );
    }
    let my_message = "Hi Sofia SIP\n";

    let handle = {
        let tags = TagBuilder::default()
            .tag(Tag::SipTo(nua_b_url).unwrap())
            .tag(Tag::NuUrl(nua_b_url).unwrap())
            .collect();
        Handle::create(&nua_a, tags).unwrap()
    };

    let tags = TagBuilder::default()
        .tag(Tag::SipSubject("NUA").unwrap())
        .tag(Tag::SipTo(nua_b_url).unwrap())
        .tag(Tag::NuUrl(nua_b_url).unwrap())
        .tag(Tag::SipContentType("text/plain").unwrap())
        .tag(Tag::SipPayloadString(my_message).unwrap())
        .collect();

    handle.message(tags);
    println!("--> Root run start");
    Root::get_default_root().unwrap().run();
    // Root::get_default_root().unwrap().sleep(0);
    println!("--> Root run end");

    assert_eq!(&*recv_message.borrow(), my_message);
}
