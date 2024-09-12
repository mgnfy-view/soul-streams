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
    });

    it("Can create a new stream and emits event", async () => {
        // Our stream configuration
        const amount = new anchor.BN(100 * Math.pow(10, DECIMALS)); // 100 tokens
        const startingTimestamp = new anchor.BN(Math.floor(Date.now() / 1000));
        const duration = new anchor.BN(1); // 1 second stream so that we can test results quickly

        const expectedStreamCount = 1;

        // Pre-calculate pda addresses
        const streamCountPublicKey = getStreamCountPublicKey(program.programId);
        const streamPublicKey = getStreamPublicKey(
            owner.publicKey,
            recipient.publicKey,
            mint,
            expectedStreamCount,
            program.programId
        );
        const streamTokenAccountPublicKey = getStreamTokenAccountPublicKey(
            owner.publicKey,
            recipient.publicKey,
            mint,
            expectedStreamCount,
            program.programId
        );

        // Before creating the stream, the payer should hold some amount of the token to be streamed
        await mintTo(provider.connection, owner, mint, ownerAssociatedTokenAccount, owner, amount.toNumber());

        // Listen for the stream creation event
        const createStreamEventListener = program.addEventListener("newStreamCreated", (event) => {
            assert.equal(event.stream.toString(), streamPublicKey.toString());
            assert.equal(event.payer.toString(), owner.publicKey.toString());
            assert.equal(event.payee.toString(), recipient.publicKey.toString());
            assert.equal(event.mint.toString(), mint.toString());
            assert.equal(event.amount.toNumber(), amount.toNumber());
            assert.equal(event.startingTimestamp.toNumber(), startingTimestamp.toNumber());
            assert.equal(event.duration.toNumber(), duration.toNumber());
            assert.equal(event.count.toNumber(), expectedStreamCount);
        });

        // Initialize and fund the stream
        await program.methods
            .createStream(recipient.publicKey, amount, startingTimestamp, duration)
            .accounts({
                payer: owner.publicKey,
                mint: mint.toBase58(),
            })
            .rpc();

        const streamCountAccount = await program.account.streamCount.fetch(streamCountPublicKey);
        const streamAccount = await program.account.stream.fetch(streamPublicKey);

        const streamTokenAccountBalance = +(
            await provider.connection.getTokenAccountBalance(streamTokenAccountPublicKey)
        ).value.amount;
        const payerTokenAccountBalance = +(
            await provider.connection.getTokenAccountBalance(ownerAssociatedTokenAccount)
        ).value.amount;

        // Stream count for the next stream to be created should be incremented by 1
        assert.equal(streamCountAccount.count.toNumber(), expectedStreamCount + 1);

        assert.equal(streamAccount.payer.toString(), owner.publicKey.toString());
        assert.equal(streamAccount.payee.toString(), recipient.publicKey.toString());
        assert.equal(streamAccount.mint.toString(), mint.toString());
        assert.equal(streamAccount.amount.toNumber(), amount.toNumber());
        assert.equal(streamAccount.startingTimestamp.toNumber(), startingTimestamp.toNumber());
        assert.equal(streamAccount.duration.toNumber(), duration.toNumber());
        assert.equal(streamAccount.streamedAmountSoFar.toNumber(), 0);
        assert.equal(streamAccount.count.toNumber(), expectedStreamCount);

        assert.equal(streamTokenAccountBalance, amount.toNumber());
        assert.equal(payerTokenAccountBalance, 0);

        await program.removeEventListener(createStreamEventListener);
    });

    it("Can withdraw from a valid stream and emits event", async () => {
        const streamCount = 1;
        const waitFor = 2000; // 2000 milliseconds, or 2 seconds

        const expectedStreamTokenAccountBalance = 0;
        const expectedPayeeTokenAccountBalance = 100e9;
        const expectedPayerTokenAccountBalance = 0;

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

        // Wait for 2 seconds so that we can collect the full amount from the stream
        await new Promise((resolve) => {
            setTimeout(resolve, waitFor);
        });

        // Listen and validate the withdrawal event
        const withdrawFromStreamEventListener = program.addEventListener("amountWithdrawnFromStream", (event) => {
            assert.equal(event.stream.toString(), streamPublicKey.toBase58());
            assert.equal(event.amountWithdrawn.toNumber(), expectedPayeeTokenAccountBalance);
        });

        await program.methods
            .withdrawFromStream(owner.publicKey, new anchor.BN(streamCount))
            .accounts({
                payee: recipient.publicKey,
                mint,
            })
            .signers([recipient])
            .rpc();

        // Retrieve the required pda
        const streamAccount = await program.account.stream.fetch(streamPublicKey);

        const streamTokenAccountBalance = +(
            await provider.connection.getTokenAccountBalance(streamTokenAccountPublicKey)
        ).value.amount;
        const recipientAssociatedTokenAccountBalance = +(
            await provider.connection.getTokenAccountBalance(recipientAssociatedTokenAccount)
        ).value.amount;
        const ownerAssociatedTokenAccountBalance = +(
            await provider.connection.getTokenAccountBalance(ownerAssociatedTokenAccount)
        ).value.amount;

        assert.equal(streamTokenAccountBalance, expectedStreamTokenAccountBalance);
        assert.equal(recipientAssociatedTokenAccountBalance, expectedPayeeTokenAccountBalance);
        assert.equal(ownerAssociatedTokenAccountBalance, expectedPayerTokenAccountBalance);

        assert.equal(streamAccount.streamedAmountSoFar.toNumber(), expectedPayeeTokenAccountBalance);

        // Remove the withdrawal event listener
        await program.removeEventListener(withdrawFromStreamEventListener);
    });
});
