import * as anchor from "@coral-xyz/anchor";

import { seedStrings } from "./constants";

function getStreamCountPublicKey(programId: anchor.web3.PublicKey): anchor.web3.PublicKey {
    return anchor.web3.PublicKey.findProgramAddressSync([Buffer.from(seedStrings.streamCount)], programId)[0];
}

function getStreamPublicKey(
    payer: anchor.web3.PublicKey,
    payee: anchor.web3.PublicKey,
    mint: anchor.web3.PublicKey,
    count: number,
    programId: anchor.web3.PublicKey
): anchor.web3.PublicKey {
    return anchor.web3.PublicKey.findProgramAddressSync(
        [
            Buffer.from(seedStrings.stream),
            payer.toBuffer(),
            payee.toBuffer(),
            mint.toBuffer(),
            new anchor.BN(count).toArrayLike(Buffer, "le", 8),
        ],
        programId
    )[0];
}

function getStreamTokenAccountPublicKey(
    payer: anchor.web3.PublicKey,
    payee: anchor.web3.PublicKey,
    mint: anchor.web3.PublicKey,
    count: number,
    programId: anchor.web3.PublicKey
): anchor.web3.PublicKey {
    return anchor.web3.PublicKey.findProgramAddressSync(
        [
            Buffer.from(seedStrings.tokenAccount),
            payer.toBuffer(),
            payee.toBuffer(),
            mint.toBuffer(),
            new anchor.BN(count).toArrayLike(Buffer, "le", 8),
        ],
        programId
    )[0];
}

export { getStreamCountPublicKey, getStreamPublicKey, getStreamTokenAccountPublicKey };
