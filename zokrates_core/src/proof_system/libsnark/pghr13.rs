use ir;
use proof_system::libsnark::ffi::{Buffer, ProofResult, SetupResult};
use proof_system::libsnark::{prepare_generate_proof, prepare_public_inputs, prepare_setup};
use proof_system::solidity::{
    SOLIDITY_G2_ADDITION_LIB, SOLIDITY_PAIRING_LIB, SOLIDITY_PAIRING_LIB_V2,
};
use proof_system::{G1Affine, G2Affine, Proof, ProofSystem, SetupKeypair, SolidityAbi};
use regex::Regex;

use zokrates_field::Bn128Field;
use zokrates_field::Field;

pub struct PGHR13 {}

#[derive(Serialize, Deserialize)]
pub struct VerificationKey {
    a: G2Affine,
    b: G1Affine,
    c: G2Affine,
    gamma: G2Affine,
    gamma_beta_1: G1Affine,
    gamma_beta_2: G2Affine,
    z: G2Affine,
    ic: Vec<G1Affine>,
    raw: String,
}

#[derive(Serialize, Deserialize)]
pub struct ProofPoints {
    a: G1Affine,
    a_p: G1Affine,
    b: G2Affine,
    b_p: G1Affine,
    c: G1Affine,
    c_p: G1Affine,
    h: G1Affine,
    k: G1Affine,
}

extern "C" {
    fn pghr13_bn128_setup(
        a: *const u8,
        b: *const u8,
        c: *const u8,
        a_len: i32,
        b_len: i32,
        c_len: i32,
        constraints: i32,
        variables: i32,
        inputs: i32,
    ) -> SetupResult;

    fn pghr13_bn128_generate_proof(
        pk_buf: *mut Buffer,
        public_query_inputs: *const u8,
        public_query_inputs_length: i32,
        private_inputs: *const u8,
        private_inputs_length: i32,
    ) -> ProofResult;

    fn pghr13_bn128_verify(
        vk_buf: *mut Buffer,
        proof_buf: *mut Buffer,
        public_inputs: *const u8,
        public_inputs_length: i32,
    ) -> bool;
}

impl ProofSystem<Bn128Field> for PGHR13 {
    type VerificationKey = VerificationKey;
    type ProofPoints = ProofPoints;

    fn setup(program: ir::Prog<Bn128Field>) -> SetupKeypair<VerificationKey> {
        let (a_arr, b_arr, c_arr, a_vec, b_vec, c_vec, num_constraints, num_variables, num_inputs) =
            prepare_setup(program);

        let keypair = unsafe {
            let result: SetupResult = pghr13_bn128_setup(
                a_arr.as_ptr(),
                b_arr.as_ptr(),
                c_arr.as_ptr(),
                a_vec.len() as i32,
                b_vec.len() as i32,
                c_vec.len() as i32,
                num_constraints as i32,
                num_variables as i32,
                num_inputs as i32,
            );

            let vk: Vec<u8> =
                std::slice::from_raw_parts(result.vk.data, result.vk.length as usize).to_vec();
            let pk: Vec<u8> =
                std::slice::from_raw_parts(result.pk.data, result.pk.length as usize).to_vec();

            // Memory is allocated in C and raw pointers are returned to Rust. The caller has to manually
            // free the memory.
            result.vk.free();
            result.pk.free();

            (vk, pk)
        };

        let vk = serde_json::from_str(String::from_utf8(keypair.0).unwrap().as_str()).unwrap();
        SetupKeypair::new(vk, keypair.1)
    }

    fn generate_proof(
        program: ir::Prog<Bn128Field>,
        witness: ir::Witness<Bn128Field>,
        proving_key: Vec<u8>,
    ) -> Proof<ProofPoints> {
        let (public_inputs_arr, public_inputs_length, private_inputs_arr, private_inputs_length) =
            prepare_generate_proof(program, witness);

        let mut pk_buf = Buffer::from_vec(&proving_key);

        let proof = unsafe {
            let result = pghr13_bn128_generate_proof(
                &mut pk_buf as *mut _,
                public_inputs_arr[0].as_ptr(),
                public_inputs_length as i32,
                private_inputs_arr[0].as_ptr(),
                private_inputs_length as i32,
            );

            pk_buf.drop(); // drop the buffer manually

            let proof: Vec<u8> =
                std::slice::from_raw_parts(result.proof.data, result.proof.length as usize)
                    .to_vec();

            // Memory is allocated in C and raw pointers are returned to Rust. The caller has to manually
            // free the memory.
            result.proof.free();

            proof
        };

        serde_json::from_str(String::from_utf8(proof).unwrap().as_str()).unwrap()
    }

    fn export_solidity_verifier(vk: VerificationKey, abi: SolidityAbi) -> String {
        let (mut template_text, solidity_pairing_lib) = match abi {
            SolidityAbi::V1 => (
                String::from(CONTRACT_TEMPLATE),
                String::from(SOLIDITY_PAIRING_LIB),
            ),
            SolidityAbi::V2 => (
                String::from(CONTRACT_TEMPLATE_V2),
                String::from(SOLIDITY_PAIRING_LIB_V2),
            ),
        };

        // replace things in template
        let vk_regex = Regex::new(r#"(<%vk_[^i%]*%>)"#).unwrap();
        let vk_ic_len_regex = Regex::new(r#"(<%vk_ic_length%>)"#).unwrap();
        let vk_ic_repeat_regex = Regex::new(r#"(<%vk_ic_pts%>)"#).unwrap();
        let vk_input_len_regex = Regex::new(r#"(<%vk_input_length%>)"#).unwrap();

        template_text = vk_regex
            .replace(template_text.as_str(), vk.a.to_string().as_str())
            .into_owned();

        template_text = vk_regex
            .replace(template_text.as_str(), vk.b.to_string().as_str())
            .into_owned();

        template_text = vk_regex
            .replace(template_text.as_str(), vk.c.to_string().as_str())
            .into_owned();

        template_text = vk_regex
            .replace(template_text.as_str(), vk.gamma.to_string().as_str())
            .into_owned();

        template_text = vk_regex
            .replace(template_text.as_str(), vk.gamma_beta_1.to_string().as_str())
            .into_owned();

        template_text = vk_regex
            .replace(template_text.as_str(), vk.gamma_beta_2.to_string().as_str())
            .into_owned();

        template_text = vk_regex
            .replace(template_text.as_str(), vk.z.to_string().as_str())
            .into_owned();

        let ic_count: usize = vk.ic.len();
        template_text = vk_ic_len_regex
            .replace(template_text.as_str(), format!("{}", ic_count).as_str())
            .into_owned();

        template_text = vk_input_len_regex
            .replace(template_text.as_str(), format!("{}", ic_count - 1).as_str())
            .into_owned();

        let mut ic_repeat_text = String::new();
        for (i, g1) in vk.ic.iter().enumerate() {
            ic_repeat_text.push_str(
                format!(
                    "vk.ic[{}] = Pairing.G1Point({});",
                    i,
                    g1.to_string().as_str()
                )
                .as_str(),
            );
            if i < ic_count - 1 {
                ic_repeat_text.push_str("\n        ");
            }
        }

        template_text = vk_ic_repeat_regex
            .replace(template_text.as_str(), ic_repeat_text.as_str())
            .into_owned();

        let re = Regex::new(r"(?P<v>0[xX][0-9a-fA-F]{64})").unwrap();
        template_text = re.replace_all(&template_text, "uint256($v)").to_string();

        format!(
            "{}{}{}",
            SOLIDITY_G2_ADDITION_LIB, solidity_pairing_lib, template_text
        )
    }

    fn verify(vk: VerificationKey, proof: Proof<ProofPoints>) -> bool {
        let vk_raw = hex::decode(vk.raw.clone()).unwrap();
        let proof_raw = hex::decode(proof.raw.clone()).unwrap();

        let public_inputs: Vec<_> = proof
            .inputs
            .iter()
            .map(|v| {
                Bn128Field::try_from_str(v.as_str().trim_start_matches("0x"), 16)
                    .expect(format!("Invalid bn128 value: {}", v.as_str()).as_str())
            })
            .collect();

        let (public_inputs_arr, public_inputs_length) = prepare_public_inputs(public_inputs);

        let mut vk_buffer = Buffer::from_vec(&vk_raw);
        let mut proof_buffer = Buffer::from_vec(&proof_raw);

        unsafe {
            let ans = pghr13_bn128_verify(
                &mut vk_buffer as *mut _,
                &mut proof_buffer as *mut _,
                public_inputs_arr[0].as_ptr(),
                public_inputs_length as i32,
            );

            vk_buffer.drop();
            proof_buffer.drop();

            ans
        }
    }
}

const CONTRACT_TEMPLATE_V2: &str = r#"contract Verifier {
    using Pairing for *;
    struct VerifyingKey {
        Pairing.G2Point a;
        Pairing.G1Point b;
        Pairing.G2Point c;
        Pairing.G2Point gamma;
        Pairing.G1Point gamma_beta_1;
        Pairing.G2Point gamma_beta_2;
        Pairing.G2Point z;
        Pairing.G1Point[] ic;
    }
    struct Proof {
        Pairing.G1Point a;
        Pairing.G1Point a_p;
        Pairing.G2Point b;
        Pairing.G1Point b_p;
        Pairing.G1Point c;
        Pairing.G1Point c_p;
        Pairing.G1Point k;
        Pairing.G1Point h;
    }
    function verifyingKey() pure internal returns (VerifyingKey memory vk) {
        vk.a = Pairing.G2Point(<%vk_a%>);
        vk.b = Pairing.G1Point(<%vk_b%>);
        vk.c = Pairing.G2Point(<%vk_c%>);
        vk.gamma = Pairing.G2Point(<%vk_g%>);
        vk.gamma_beta_1 = Pairing.G1Point(<%vk_gb1%>);
        vk.gamma_beta_2 = Pairing.G2Point(<%vk_gb2%>);
        vk.z = Pairing.G2Point(<%vk_z%>);
        vk.ic = new Pairing.G1Point[](<%vk_ic_length%>);
        <%vk_ic_pts%>
    }
    function verify(uint[] memory input, Proof memory proof) internal returns (uint) {
        uint256 snark_scalar_field = 21888242871839275222246405745257275088548364400416034343698204186575808495617;
        VerifyingKey memory vk = verifyingKey();
        require(input.length + 1 == vk.ic.length);
        // Compute the linear combination vk_x
        Pairing.G1Point memory vk_x = Pairing.G1Point(0, 0);
        for (uint i = 0; i < input.length; i++) {
            require(input[i] < snark_scalar_field);
            vk_x = Pairing.addition(vk_x, Pairing.scalar_mul(vk.ic[i + 1], input[i]));
        }
        vk_x = Pairing.addition(vk_x, vk.ic[0]);
        if (!Pairing.pairingProd2(proof.a, vk.a, Pairing.negate(proof.a_p), Pairing.P2())) return 1;
        if (!Pairing.pairingProd2(vk.b, proof.b, Pairing.negate(proof.b_p), Pairing.P2())) return 2;
        if (!Pairing.pairingProd2(proof.c, vk.c, Pairing.negate(proof.c_p), Pairing.P2())) return 3;
        if (!Pairing.pairingProd3(
            proof.k, vk.gamma,
            Pairing.negate(Pairing.addition(vk_x, Pairing.addition(proof.a, proof.c))), vk.gamma_beta_2,
            Pairing.negate(vk.gamma_beta_1), proof.b
        )) return 4;
        if (!Pairing.pairingProd3(
                Pairing.addition(vk_x, proof.a), proof.b,
                Pairing.negate(proof.h), vk.z,
                Pairing.negate(proof.c), Pairing.P2()
        )) return 5;
        return 0;
    }
    event Verified(string s);
    function verifyTx(
            Proof memory proof,
            uint[<%vk_input_length%>] memory input
        ) public returns (bool r) {
        uint[] memory inputValues = new uint[](input.length);
        for(uint i = 0; i < input.length; i++){
            inputValues[i] = input[i];
        }
        if (verify(inputValues, proof) == 0) {
            emit Verified("Transaction successfully verified.");
            return true;
        } else {
            return false;
        }
    }
}
"#;

const CONTRACT_TEMPLATE: &str = r#"contract Verifier {
    using Pairing for *;
    struct VerifyingKey {
        Pairing.G2Point a;
        Pairing.G1Point b;
        Pairing.G2Point c;
        Pairing.G2Point gamma;
        Pairing.G1Point gamma_beta_1;
        Pairing.G2Point gamma_beta_2;
        Pairing.G2Point z;
        Pairing.G1Point[] ic;
    }
    struct Proof {
        Pairing.G1Point a;
        Pairing.G1Point a_p;
        Pairing.G2Point b;
        Pairing.G1Point b_p;
        Pairing.G1Point c;
        Pairing.G1Point c_p;
        Pairing.G1Point k;
        Pairing.G1Point h;
    }
    function verifyingKey() pure internal returns (VerifyingKey memory vk) {
        vk.a = Pairing.G2Point(<%vk_a%>);
        vk.b = Pairing.G1Point(<%vk_b%>);
        vk.c = Pairing.G2Point(<%vk_c%>);
        vk.gamma = Pairing.G2Point(<%vk_g%>);
        vk.gamma_beta_1 = Pairing.G1Point(<%vk_gb1%>);
        vk.gamma_beta_2 = Pairing.G2Point(<%vk_gb2%>);
        vk.z = Pairing.G2Point(<%vk_z%>);
        vk.ic = new Pairing.G1Point[](<%vk_ic_length%>);
        <%vk_ic_pts%>
    }
    function verify(uint[] memory input, Proof memory proof) internal returns (uint) {
        uint256 snark_scalar_field = 21888242871839275222246405745257275088548364400416034343698204186575808495617;
        VerifyingKey memory vk = verifyingKey();
        require(input.length + 1 == vk.ic.length);
        // Compute the linear combination vk_x
        Pairing.G1Point memory vk_x = Pairing.G1Point(0, 0);
        for (uint i = 0; i < input.length; i++) {
            require(input[i] < snark_scalar_field);
            vk_x = Pairing.addition(vk_x, Pairing.scalar_mul(vk.ic[i + 1], input[i]));
        }
        vk_x = Pairing.addition(vk_x, vk.ic[0]);
        if (!Pairing.pairingProd2(proof.a, vk.a, Pairing.negate(proof.a_p), Pairing.P2())) return 1;
        if (!Pairing.pairingProd2(vk.b, proof.b, Pairing.negate(proof.b_p), Pairing.P2())) return 2;
        if (!Pairing.pairingProd2(proof.c, vk.c, Pairing.negate(proof.c_p), Pairing.P2())) return 3;
        if (!Pairing.pairingProd3(
            proof.k, vk.gamma,
            Pairing.negate(Pairing.addition(vk_x, Pairing.addition(proof.a, proof.c))), vk.gamma_beta_2,
            Pairing.negate(vk.gamma_beta_1), proof.b
        )) return 4;
        if (!Pairing.pairingProd3(
                Pairing.addition(vk_x, proof.a), proof.b,
                Pairing.negate(proof.h), vk.z,
                Pairing.negate(proof.c), Pairing.P2()
        )) return 5;
        return 0;
    }
    event Verified(string s);
    function verifyTx(
            uint[2] memory a,
            uint[2] memory a_p,
            uint[2][2] memory b,
            uint[2] memory b_p,
            uint[2] memory c,
            uint[2] memory c_p,
            uint[2] memory h,
            uint[2] memory k,
            uint[<%vk_input_length%>] memory input
        ) public returns (bool r) {
        Proof memory proof;
        proof.a = Pairing.G1Point(a[0], a[1]);
        proof.a_p = Pairing.G1Point(a_p[0], a_p[1]);
        proof.b = Pairing.G2Point([b[0][0], b[0][1]], [b[1][0], b[1][1]]);
        proof.b_p = Pairing.G1Point(b_p[0], b_p[1]);
        proof.c = Pairing.G1Point(c[0], c[1]);
        proof.c_p = Pairing.G1Point(c_p[0], c_p[1]);
        proof.h = Pairing.G1Point(h[0], h[1]);
        proof.k = Pairing.G1Point(k[0], k[1]);
        uint[] memory inputValues = new uint[](input.length);
        for(uint i = 0; i < input.length; i++){
            inputValues[i] = input[i];
        }
        if (verify(inputValues, proof) == 0) {
            emit Verified("Transaction successfully verified.");
            return true;
        } else {
            return false;
        }
    }
}
"#;
