import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { mintTo } from "@solana/spl-token";
import { assert } from "chai";
import { SoulStreams } from "../target/types/soul_streams";

import { setup } from "./utils/setup";
import { DECIMALS, seedStrings } from "./utils/constants";
import { getStreamCountPublicKey, getStreamPublicKey, getStreamTokenAccountPublicKey } from "./utils/utils";

describe("Soul Streams", () => {
    let provider: anchor.Provider;
    let program: Program<SoulStreams>;

    let owner: anchor.web3.Keypair;
    let recipient: anchor.web3.Keypair;

    let mint: anchor.web3.PublicKey;

    let ownerAssociatedTokenAccount: anchor.web3.PublicKey;
    let recipientAssociatedTokenAccount: anchor.web3.PublicKey;

    before(async () => {
        // Set up a provider and the program. Get the owner's and recipient's keypairs, get the mint, as well
        // as the owner and recipient's associated token accounts for the mint
        ({ provider, program, owner, recipient, mint, ownerAssociatedTokenAccount, recipientAssociatedTokenAccount } =
            await setup());

        // Initialize the stream count to 1
        await program.methods.initialize().rpc();

        // Our new stream's configuration
        const amount = new anchor.BN(100 * Math.pow(10, DECIMALS)); // 100 tokens
        const startingTimestamp = new anchor.BN(Math.floor(Date.now() / 1000));
        const duration = new anchor.BN(1); // 1 second stream so that we can test results quickly

        // Before creating the stream, the payer should hold some amount of the token to be streamed
        await mintTo(provider.connection, owner, mint, ownerAssociatedTokenAccount, owner, amount.toNumber());

        // Initialize and fund the stream
        await program.methods
            .createStream(recipient.publicKey, amount, startingTimestamp, duration)
            .accounts({
                payer: owner.publicKey,
                mint: mint.toBase58(),
            })
            .rpc();
    });

    it("Can cancel a stream and emits event", async () => {
        const streamCount = 1;

        // Pre-calculate pda addresses
        const streamPublicKey = getStreamPublicKey(
            owner.publicKey,
            recipient.publicKey,
            mint,
            streamCount,
            program.programId
        );
        const streamTokenAccountPublicKey = getStreamTokenAccountPublicKey(
            owner.publicKey,
            recipient.publicKey,
            mint,
            streamCount,
            program.programId
        );

        // Listen for the stream cancellation event
        const streamCancelledEventListener = program.addEventListener("streamCanceled", (event) => {
            assert.equal(event.stream.toString(), streamPublicKey.toString());
            assert.equal(event.payer.toString(), owner.publicKey.toString());
            assert.equal(event.payee.toString(), recipient.publicKey.toString());
            assert.equal(event.mint.toString(), mint.toString());
            assert.equal(event.count.toNumber(), streamCount);
        });

        await program.methods
            .cancelStream(recipient.publicKey, new anchor.BN(streamCount))
            .accounts({
                payer: owner.publicKey,
                mint: mint.toBase58(),
            })
            .rpc();

        // Check if the payer correctly got his tokens back from the stream
        const streamTokenAccountBalance = +(
            await provider.connection.getTokenAccountBalance(streamTokenAccountPublicKey)
        ).value.amount;
        const ownerAssociatedTokenAccountBalance = +(
            await provider.connection.getTokenAccountBalance(ownerAssociatedTokenAccount)
        ).value.amount;
        const recipientAssociatedTokenAccountBalance = +(
            await provider.connection.getTokenAccountBalance(recipientAssociatedTokenAccount)
        ).value.amount;

        const expectedStreamTokenAccountBalance = 0;
        const expectedPayerTokenAccountBalance = 100e9;
        const expectedPayeeTokenAccountBalance = 0;

        assert.equal(streamTokenAccountBalance, expectedStreamTokenAccountBalance);
        assert.equal(ownerAssociatedTokenAccountBalance, expectedPayerTokenAccountBalance);
        assert.equal(recipientAssociatedTokenAccountBalance, expectedPayeeTokenAccountBalance);

        // Since the stream account has been closed, fetching it should throw an error
        try {
            await program.account.stream.fetch(streamPublicKey);
        } catch (err) {
            assert((err as Error).message);
        }

        await program.removeEventListener(streamCancelledEventListener);
    });
});
