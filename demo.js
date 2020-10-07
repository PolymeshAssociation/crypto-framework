// ALICE: 0x0000000000000000000000000000000006000000000000000000000000000000
// ALD_1: 0x406e673be729d19ef4b04ff41456863e247a708b7433078e4663a90e08af8026
// ALD_2: 0xc0a17b7c73f7a14b9d59666a0e161c8087f74bc45b0a9711dbbcf186d909cf57
// ALD_3: 0x650456f40d5d67f29f522631903d43140c4eeb85e7fa0616cb6e837dc3609d39

// ALICE_UID: [94, 58, 182, 27, 30, 137, 81, 212, 254, 154, 230, 123, 171, 97, 74, 95]
// ALD_1_UID: [152, 25, 31, 70, 229, 131, 2, 22, 68, 84, 54, 151, 136, 3, 105, 122]
// ALD_2_UID: [152, 25, 31, 70, 229, 131, 2, 22, 68, 84, 54, 151, 136, 3, 105, 122]
// ALD_3_UID: [123, 252, 253, 117, 68, 177, 7, 141, 218, 57, 124, 239, 69, 223, 46, 109]

function parseHexString(str) { 
    var result = [];
    while (str.length >= 2) { 
        result.push(parseInt(str.substring(0, 2), 16));
        str = str.substring(2, str.length);
    }

    return result;
}

function createHexString(arr) {
    var result = "";
    for (i in arr) {
        var str = arr[i].toString(16);
        str = str.length == 0 ? "00" :
              str.length == 1 ? "0" + str : 
              str.length == 2 ? str :
              str.substring(str.length-2, str.length);
        result += str;
    }
    return result;
}

let alice = "0000000000000000000000000000000006000000000000000000000000000000"
let ald_1 = "406e673be729d19ef4b04ff41456863e247a708b7433078e4663a90e08af8026"
let ald_2 = "c0a17b7c73f7a14b9d59666a0e161c8087f74bc45b0a9711dbbcf186d909cf57"
let ald_3 = "650456f40d5d67f29f522631903d43140c4eeb85e7fa0616cb6e837dc3609d39"

let uid_alice_arr = [94, 58, 182, 27, 30, 137, 81, 212, 254, 154, 230, 123, 171, 97, 74, 95]
let uid_1_arr = [152, 25, 31, 70, 229, 131, 2, 22, 68, 84, 54, 151, 136, 3, 105, 122]
let uid_2_arr = [152, 25, 31, 70, 229, 131, 2, 22, 68, 84, 54, 151, 136, 3, 105, 122]
let uid_3_arr = [123, 252, 253, 117, 68, 177, 7, 141, 218, 57, 124, 239, 69, 223, 46, 109]

createHexString(uid_alice_arr)
// '5e3ab61b1e8951d4fe9ae67bab614a5f'
createHexString(uid_1_arr)
// '98191f46e5830216445436978803697a'
createHexString(uid_2_arr)
// '98191f46e5830216445436978803697a'
createHexString(uid_3_arr)
// '7bfcfd7544b1078dda397cef45df2e6d'

parseHexString(alice).toString()
// '0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,6,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0'
parseHexString(ald_1).toString()
// '64,110,103,59,231,41,209,158,244,176,79,244,20,86,134,62,36,122,112,139,116,51,7,142,70,99,169,14,8,175,128,38'
parseHexString(ald_2).toString()
// '192,161,123,124,115,247,161,75,157,89,102,106,14,22,28,128,135,247,75,196,91,10,151,17,219,188,241,134,217,9,207,87'
parseHexString(ald_3).toString()
// '101,4,86,244,13,93,103,242,159,82,38,49,144,61,67,20,12,78,235,133,231,250,6,22,203,110,131,125,195,96,157,57'

let acme = "41434d45"
parseHexString(acme).toString()
// '65,67,77,69'

// adamdossa@Adams-MBP crypto-framework % ./target/release/polymath-scp create-cdd-id --cdd-claim alice_cdd_claim.json --cdd-id alice_cdd_id.json -v
// CDD Id Package: "{\"cdd_id\":[76,215,94,0,123,102,21,122,9,179,173,18,51,217,133,135,72,104,249,108,197,138,254,43,2,111,20,197,243,0,190,98]}"
// Successfully wrote the CDD Id.

// adamdossa@Adams-MBP crypto-framework % ./target/release/polymath-scp create-cdd-id --cdd-claim ald_1_cdd_claim.json --cdd-id ald_1_cdd_id.json -v
// CDD Id Package: "{\"cdd_id\":[98,123,138,16,241,103,86,170,76,95,173,231,30,143,65,31,227,234,201,70,93,100,228,236,244,227,1,105,51,14,28,42]}"
// Successfully wrote the CDD Id.

// adamdossa@Adams-MBP crypto-framework % ./target/release/polymath-scp create-cdd-id --cdd-claim ald_2_cdd_claim.json --cdd-id ald_2_cdd_id.json -v
// CDD Id Package: "{\"cdd_id\":[160,212,113,151,55,215,126,233,9,28,11,28,216,114,30,89,44,21,58,201,159,58,89,218,74,146,237,87,149,0,230,29]}"
// Successfully wrote the CDD Id.

// adamdossa@Adams-MBP crypto-framework % ./target/release/polymath-scp create-cdd-id --cdd-claim ald_3_cdd_claim.json --cdd-id ald_3_cdd_id.json -v   
// CDD Id Package: "{\"cdd_id\":[44,173,135,116,49,16,200,190,113,54,254,185,223,16,13,19,167,248,152,74,159,90,92,61,85,217,30,35,93,200,62,122]}"
// Successfully wrote the CDD Id.

let cdd_alice = [76,215,94,0,123,102,21,122,9,179,173,18,51,217,133,135,72,104,249,108,197,138,254,43,2,111,20,197,243,0,190,98]
let cdd_1 = [98,123,138,16,241,103,86,170,76,95,173,231,30,143,65,31,227,234,201,70,93,100,228,236,244,227,1,105,51,14,28,42]
let cdd_2 = [56,69,247,136,89,166,108,218,130,173,246,41,65,151,14,169,199,161,132,116,255,33,159,85,184,58,11,96,137,172,251,38]
let cdd_3 = [76,10,167,232,220,39,125,73,121,70,209,17,222,132,106,251,34,170,161,48,203,233,137,143,116,53,24,116,180,97,226,8]

createHexString(cdd_alice)
// '4cd75e007b66157a09b3ad1233d985874868f96cc58afe2b026f14c5f300be62'
createHexString(cdd_1)
// '627b8a10f16756aa4c5fade71e8f411fe3eac9465d64e4ecf4e30169330e1c2a'
createHexString(cdd_2)
// '3845f78859a66cda82adf62941970ea9c7a18474ff219f55b83a0b6089acfb26'
createHexString(cdd_3)
// '4c0aa7e8dc277d497946d111de846afb22aaa130cbe9898f74351874b461e208'

// ./target/release/polymath-scp create-claim-proof -v --cdd-claim alice_cdd_claim.json --scope-claim alice_scope_claim.json --proof alice_proof.json
// ./target/release/polymath-scp create-claim-proof -v --cdd-claim ald_1_cdd_claim.json --scope-claim ald_1_scope_claim.json --proof ald_1_proof.json
// ./target/release/polymath-scp create-claim-proof -v --cdd-claim ald_2_cdd_claim.json --scope-claim ald_2_scope_claim.json --proof ald_2_proof.json
// ./target/release/polymath-scp create-claim-proof -v --cdd-claim ald_3_cdd_claim.json --scope-claim ald_3_scope_claim.json --proof ald_3_proof.json

let full_proof_alice = JSON.parse(fs.readFileSync('alice_proof.json'))
let full_proof_1 = JSON.parse(fs.readFileSync('ald_1_proof.json'))
let full_proof_2 = JSON.parse(fs.readFileSync('ald_2_proof.json'))
let full_proof_3 = JSON.parse(fs.readFileSync('ald_3_proof.json'))

createHexString(full_proof_alice['cdd_id'])
// '4cd75e007b66157a09b3ad1233d985874868f96cc58afe2b026f14c5f300be62'
createHexString(full_proof_alice['scope_id'])
// '966997c7f9ce11593ac65ced74be946c1a87cbcda6cf848cea6b0a93b5e01a12'
createHexString(full_proof_alice['proof'])
// '64bd8c26d447f82eec9007aa276a58efabbc596fef93c1147fca19d581cad62a5f6e5512e4bce94f63a469f30eea0c25169dbaa58b8df93a3d1e6d773c8b118f'

createHexString(full_proof_1['cdd_id'])
// '627b8a10f16756aa4c5fade71e8f411fe3eac9465d64e4ecf4e30169330e1c2a'
createHexString(full_proof_1['scope_id'])
// '76c0e530061738701d2b1de2143716de8bf793bd652a03b6e987e2d24c5c4319'
createHexString(full_proof_1['proof'])
// '923f273cae0d37dc45f32c7853efce027bd85def55c21ddb647ac49765031c5683d6069351f4fc442acb16c3dc1558ebd1da1aeed762826edec70dca119c568d'

createHexString(full_proof_2['cdd_id'])
// '3845f78859a66cda82adf62941970ea9c7a18474ff219f55b83a0b6089acfb26'
createHexString(full_proof_2['scope_id'])
// '76c0e530061738701d2b1de2143716de8bf793bd652a03b6e987e2d24c5c4319'
createHexString(full_proof_2['proof'])
// '70d8f29e53cb6c7fa24c046ab7f347b2af3903014654008eed0046610de94a6dd88aa79d2ff71b92704515f2d4d62f99468f9d9e48021839733268e252161b84'

createHexString(full_proof_3['cdd_id'])
// '4c0aa7e8dc277d497946d111de846afb22aaa130cbe9898f74351874b461e208'
createHexString(full_proof_3['scope_id'])
// '846c7f63dab2445838360b60c6b824c287e970f4bc4bf594080130b10de04d58'
createHexString(full_proof_3['proof'])
// '5811af0fb24e7fe204336f79b9950e17cec50378894d547793868799ad1a6d663d7f404c627f1ba2e85f46269c0e6cf0efbc3941a93a52e5f85be2326927038c'