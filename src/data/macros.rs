#[macro_export]
macro_rules! char_num {
    (0) => (30); (1) => (31); (2) => (32); (3) => (33); (4) => (34); (5) => (35);
    (6) => (36); (7) => (37); (8) => (38); (9) => (39); (A) => (65); (B) => (66);
    (C) => (67); (D) => (68); (E) => (69); (F) => (70); (G) => (71); (H) => (72);
    (I) => (73); (J) => (74); (K) => (75); (L) => (76); (M) => (77); (N) => (78);
    (O) => (79); (P) => (80); (Q) => (81); (R) => (82); (S) => (83); (T) => (84);
    (U) => (85); (V) => (86); (W) => (87); (X) => (88); (Y) => (89); (Z) => (90);
    (_) => (95);
}

#[macro_export]
macro_rules! tag {
    ($a:tt $b:tt $c:tt $d:tt) => (
        char_num!($a) << 24 |
        char_num!($b) << 16 |
        char_num!($c) << 8 |
        char_num!($d)
    );
}

#[cfg(test)]
mod tests {
    use super::super::Tag;

    #[test]
    fn tag_macro() {
        const SHAP: Tag = tag!(S H A P);
        const PATH: Tag = tag!(P A T H);

        assert_eq!(SHAP, 1397244240u32);
        assert_eq!(PATH, 1346458696u32);
    }
}
