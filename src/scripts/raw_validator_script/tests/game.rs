use super::*;
use sha2::{Digest, Sha256};

struct HashedString {
    inner: Vec<u8>,
}

impl HashedString {
    pub fn new(unhashed: &str) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(unhashed.as_bytes());
        let inner = hasher.finalize().to_vec();
        HashedString { inner }
    }
}

impl From<HashedString> for PlutusData {
    fn from(hs: HashedString) -> Self {
        let bytes = hs.inner;
        PlutusData::BoundedBytes(bytes)
    }
}

struct ClearString {
    inner: String,
}

impl ClearString {
    pub fn new(inner: &str) -> Self {
        ClearString {
            inner: inner.to_string(),
        }
    }
}

impl From<ClearString> for PlutusData {
    fn from(cs: ClearString) -> Self {
        let bytes = cs.inner.as_bytes().to_vec();
        PlutusData::BoundedBytes(bytes)
    }
}

// TODO: Broken for some reason
//  https://github.com/MitchTurner/naumachia/issues/80
#[ignore]
#[test]
fn execute_game_passes() {
    // let script_file = _game_lite_script_file();
    let script_file = game_script_file();
    let script = RawPlutusValidator::new_v1(script_file).unwrap();

    let word = "konnichiwa";

    let datum = HashedString::new(word);
    let redeemer = ClearString::new(word);
    let signer = Address::from_bech32("addr_test1qzvrhz9v6lwcr26a52y8mmk2nzq37lky68359keq3dgth4lkzpnnjv8vf98m20lhqdzl60mcftq7r2lc4xtcsv0w6xjstag0ua").unwrap();
    let ctx = ContextBuilder::new(signer).build_spend(&vec![], 0);

    assert!(dbg!(script.execute(datum, redeemer, ctx)).is_ok());
}

// TODO: Broken for some reason
//  https://github.com/MitchTurner/naumachia/issues/80
#[ignore]
#[test]
fn execute_game_fails() {
    // let script_file = _game_lite_script_file();
    let script_file = game_script_file();
    let script = RawPlutusValidator::new_v1(script_file).unwrap();

    let word = "konnichiwa";
    let bad_guess = "kombanwa";

    let datum = HashedString::new(word);
    let redeemer = ClearString::new(bad_guess);
    let signer = Address::from_bech32("addr_test1qzvrhz9v6lwcr26a52y8mmk2nzq37lky68359keq3dgth4lkzpnnjv8vf98m20lhqdzl60mcftq7r2lc4xtcsv0w6xjstag0ua").unwrap();
    let ctx = ContextBuilder::new(signer).build_spend(&vec![], 0);

    assert!(dbg!(script.execute(datum, redeemer, ctx)).is_err()); // TODO: Make more specific
}

// TODO: Broken
// error: Err(EvaluationFailure)
// logs: []
fn _game_lite_script_file() -> PlutusScriptFile {
    PlutusScriptFile {
        r#type: "PlutusScriptV1".to_string(),
        description: "".to_string(),
        // The ScriptContext is just two `BuiltinData`s
        cborHex: "5840583e0100003232222533532323232333573466e3c010004488008488004dc900118030019bae\
        003375c006200a264c649319ab9c490103505435000056120011"
            .to_string(),
        // Works (Replace all of ScriptContext with BuiltinData)
        // cborHex: "583a5838010000322225335323232333573466e3c00c004488008488004dc90009bae003375c0062008264c649319ab9c49103505435000041200101"
        //     .to_string(),
    }
}

// TODO: Broken with a different error
// error: Err(EmptyList(Con(ProtoList(Data, []))))
// logs: []
fn game_script_file() -> PlutusScriptFile {
    PlutusScriptFile {
        r#type: "PlutusScriptV1".to_string(),
        description: "".to_string(),
        cborHex: "59072d59072a01000032323232323232323233223232323232332232323232222323253353232323\
        2333573466e3c010004058054dc90011999ab9a3370e6aae754011200023322123300100300232323232323232\
        323232323333573466e1cd55cea8052400046666666666444444444424666666666600201601401201000e00c0\
        0a00800600466a02a464646666ae68cdc39aab9d5002480008cc8848cc00400c008c080d5d0a801180d1aba135\
        744a004464c6405666ae700b40ac0a44d55cf280089baa00135742a01466a02a02c6ae854024ccd54061d7280b\
        9aba150083335501875ca02e6ae85401ccd4054088d5d0a80319a80a99aa812811bad35742a00a6464646666ae\
        68cdc39aab9d5002480008cc8848cc00400c008c8c8c8cccd5cd19b8735573aa00490001199109198008018011\
        9a8133ad35742a004604e6ae84d5d1280111931901799ab9c03102f02d135573ca00226ea8004d5d0a80119191\
        91999ab9a3370e6aae754009200023322123300100300233502675a6ae854008c09cd5d09aba2500223263202f\
        33573806205e05a26aae7940044dd50009aba135744a004464c6405666ae700b40ac0a44d55cf280089baa0013\
        5742a00866a02aeb8d5d0a80199a80a99aa812bae200135742a004603a6ae84d5d1280111931901399ab9c0290\
        27025135744a00226ae8940044d5d1280089aba25001135744a00226ae8940044d5d1280089aba25001135573c\
        a00226ea8004d5d0a8021919191999ab9a3370ea0029003119091111802002980d1aba135573ca00646666ae68\
        cdc3a8012400846424444600400a60386ae84d55cf280211999ab9a3370ea0069001119091111800802980b1ab\
        a135573ca00a46666ae68cdc3a8022400046424444600600a6eb8d5d09aab9e500623263202233573804804404\
        003e03c03a26aae7540044dd50009aba135744a008464c6403666ae7007406c064dd70029bae00510181326320\
        1833573892010350543500018135573ca00226ea800448c88c008dd6000990009aa80b111999aab9f001250092\
        33500830043574200460066ae880080548c8c8c8cccd5cd19b8735573aa0069000119991109199800802001801\
        1919191999ab9a3370e6aae7540092000233221233001003002301735742a00466a01c02c6ae84d5d128011193\
        1900d19ab9c01c01a018135573ca00226ea8004d5d0a801999aa803bae500635742a00466a014eb8d5d09aba25\
        00223263201633573803002c02826ae8940044d55cf280089baa0011335500175ceb44488c88c008dd58009900\
        09aa80a11191999aab9f0022500823350073355016300635573aa004600a6aae794008c010d5d100180a09aba1\
        00111220021221223300100400312232323333573466e1d4005200023212230020033005357426aae79400c8cc\
        cd5cd19b8750024800884880048c98c8048cd5ce00a00900800789aab9d500113754002464646666ae68cdc39a\
        ab9d5002480008cc8848cc00400c008c014d5d0a8011bad357426ae8940088c98c803ccd5ce00880780689aab9\
        e5001137540024646666ae68cdc39aab9d5001480008dd71aba135573ca004464c6401a66ae7003c03402c4dd5\
        00089119191999ab9a3370ea00290021091100091999ab9a3370ea00490011190911180180218031aba135573c\
        a00846666ae68cdc3a801a400042444004464c6402066ae700480400380340304d55cea80089baa00123233335\
        73466e1d40052002200523333573466e1d40092000200523263200c33573801c01801401226aae74dd50008910\
        01091000919191919191999ab9a3370ea002900610911111100191999ab9a3370ea00490051091111110021199\
        9ab9a3370ea00690041199109111111198008048041bae35742a00a6eb4d5d09aba2500523333573466e1d4011\
        2006233221222222233002009008375c6ae85401cdd71aba135744a00e46666ae68cdc3a802a40084664424444\
        4446600c01201060186ae854024dd71aba135744a01246666ae68cdc3a8032400446424444444600e010601a6a\
        e84d55cf280591999ab9a3370ea00e900011909111111180280418071aba135573ca018464c6402466ae700500\
        4804003c03803403002c0284d55cea80209aab9e5003135573ca00426aae7940044dd50009191919191999ab9a\
        3370ea002900111999110911998008028020019bad35742a0086eb4d5d0a8019bad357426ae89400c8cccd5cd1\
        9b875002480008c8488c00800cc020d5d09aab9e500623263200b33573801a01601201026aae75400c4d5d1280\
        089aab9e500113754002464646666ae68cdc3a800a400446424460020066eb8d5d09aab9e500323333573466e1\
        d400920002321223002003375c6ae84d55cf280211931900419ab9c00a008006005135573aa00226ea80044488\
        8c8c8cccd5cd19b8735573aa0049000119aa80498031aba150023005357426ae8940088c98c8020cd5ce005004\
        00309aab9e50011375400293090008891091980080180124810350543100112323001001223300330020020011"
            .to_string(),
    }
}
