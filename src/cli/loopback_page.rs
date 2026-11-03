use std::io::Write;
use std::net::TcpStream;

pub fn respond(stream: &mut TcpStream, status: &str, heading: &str, hint: &str, warn: bool) {
    let body = page(heading, hint, warn);
    let head = format!(
        "HTTP/1.1 {status}\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\n\
         Cache-Control: no-store\r\nConnection: close\r\n\r\n",
        body.len()
    );
    let _ = stream.write_all(head.as_bytes());
    let _ = stream.write_all(body.as_bytes());
    let _ = stream.flush();
}

fn page(heading: &str, hint: &str, warn: bool) -> String {
    let tone = if warn { "#b45309" } else { "inherit" };
    format!(
        "<!doctype html><html lang=\"en\"><meta charset=\"utf-8\">\
         <meta name=\"viewport\" content=\"width=device-width,initial-scale=1\">\
         <title>{heading} \u{b7} Opus</title>\
         <style>:root{{color-scheme:light dark}}\
         body{{margin:0;min-height:100dvh;display:flex;align-items:center;justify-content:center;\
         padding:1.5rem;font:16px/1.6 system-ui,sans-serif;background:Canvas;color:CanvasText}}\
         main{{max-width:26rem;width:100%;text-align:center}}\
         .card{{padding:1.75rem;border-radius:.75rem;background:color-mix(in srgb,CanvasText 6%,Canvas)}}\
         h1{{margin:0;font-size:1.25rem}}\
         p{{margin:.5rem 0 0;font-size:.875rem;opacity:.7;color:{tone}}}\
         .brand{{font-weight:650;margin-bottom:2rem}}</style>\
         <main><div class=\"brand\">Opus</div><div class=\"card\"><h1>{heading}</h1><p>{hint}</p></div></main></html>"
    )
}
