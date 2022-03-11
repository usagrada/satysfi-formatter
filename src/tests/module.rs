use super::test_tmpl;
use crate::format;
use dirs;
use std::fs;

#[test]
fn module_deco() {
    let satypkg = format!(
        "{}{}",
        dirs::home_dir().unwrap().to_str().unwrap(),
        "/.satysfi/dist/packages"
    );
    let text = fs::read(format!("{satypkg}/deco.satyh")).unwrap();
    let input = String::from_utf8(text).unwrap();
    let expect = r#"@require: gr

module Deco: sig
    val empty: deco
    val simple-frame: length -> color -> color -> deco
end = struct
    let empty _ _ _ _ = []

    let simple-frame t scolor fcolor (x, y) w h d =
        let path = Gr.rectangle (x, y -' d) (x +' w, y +' h) in
        [
            fill fcolor path;
            stroke t scolor path;
        ]
end
"#;
    test_tmpl(&input, expect)
}

#[test]
fn module_table() {
    let satypkg = format!(
        "{}{}",
        dirs::home_dir().unwrap().to_str().unwrap(),
        "/.satysfi/dist/packages"
    );
    let text = fs::read(format!("{satypkg}/table.satyh")).unwrap();
    let input = String::from_utf8(text).unwrap();
    let expect = r#"module Table : sig
    direct \tabular :  [
        (|
            l : inline-text -> cell;
            r : inline-text -> cell;
            c : inline-text -> cell;
            m : int -> int -> inline-text -> cell;
            e : cell;
        |) -> (cell list) list;
        length list -> length list -> graphics list;
    ] inline-cmd
end = struct
    let table-scheme ctx pads cellssf decof =
        let nc ib = NormalCell(pads, ib) in
        let mc i j ib = MultiCell(i, j, pads, ib) in
        let cellss =
            cellssf (|
            l = (fun it ->
                    nc (read-inline ctx it ++ inline-fil));
            r = (fun it ->
                    nc (inline-fil ++ read-inline ctx it));
            c = (fun it ->
                    nc (inline-fil ++ read-inline ctx it ++ inline-fil));
            m = (fun i j it ->
                    mc i j (inline-fil ++ read-inline ctx it ++ inline-fil));
            e = EmptyCell;
            |)
        in tabular cellss decof

    let-inline ctx \tabular =
        let pads = (2pt, 2pt, 2pt, 2pt) in table-scheme ctx pads
end
"#;
    test_tmpl(&input, expect)
}

#[test]
fn test_option() {
    let satypkg = format!(
        "{}{}",
        dirs::home_dir().unwrap().to_str().unwrap(),
        "/.satysfi/dist/packages"
    );
    let text = fs::read(format!("{satypkg}/option.satyg")).unwrap();
    let input = String::from_utf8(text).unwrap();
    let output = format(&input);
    let expect = r#"@stage: persistent

module Option: sig
    val map : ('a -> 'b) -> 'a option -> 'b option
    val from : 'a -> 'a option -> 'a
    val bind : 'a option -> ('a -> 'b option) -> 'b option
    val is-none : 'a option -> bool
end = struct
    let-rec map
        | f (None)    = None
        | f (Some(v)) = Some(f v)

    let-rec from
        | a (None)    = a
        | _ (Some(a)) = a

    let-rec bind
        | (None)    f = None
        | (Some(v)) f = f v

    let is-none opt =
        match opt with
        | None    -> true
        | Some(_) -> false
end
"#;
    assert_eq!(output, expect);
}
