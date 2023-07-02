import { Keypair } from '@solana/web3.js';
import * as anchor from '@coral-xyz/anchor';
import { user } from './config';
import fs from 'fs';

const fileName = __dirname + '/keys.json';

//SET CORRECT USER FOR TESTS IN THE LAST FUNCTION

interface Config {
  protocolSecret: Uint8Array,
  mintAuthSecret: Uint8Array,
  repMintSecret: Uint8Array,
  disputeId: string, //BN toJSON()
  decimals: number,
  userOneSecret: Uint8Array,
  userTwoSecret: Uint8Array,
  voterSecret: Uint8Array
}

export function setMint(protocol: Keypair, auth: Keypair, mint: Keypair, dec: number) {
    console.log("PROTOCOL SECRET: ", protocol.secretKey);
    console.log("MINT AUTH SECRET: ", auth.secretKey);
    console.log("REP MINT SECRET: ", mint.secretKey);
    console.log("Decimals: ", dec);

    const config: Config = {
      protocolSecret: protocol.secretKey,
      mintAuthSecret: auth.secretKey,
      repMintSecret: mint.secretKey,
      disputeId: "",
      decimals: dec,
      userOneSecret: new Uint8Array(),
      userTwoSecret: new Uint8Array(),
      voterSecret: new Uint8Array()
    };

    fs.writeFileSync(fileName, JSON.stringify(config, null, 2));
}

export function createUsers(userOne: Keypair, userTwo: Keypair, userThree: Keypair) {
    console.log("User 1: ", userOne.secretKey);
    console.log("User 2: ", userTwo.secretKey);
    console.log("User 3: ", userThree.secretKey);

    const configContents = fs.readFileSync(fileName, 'utf8');

    const config: Config = JSON.parse(configContents);
    config.userOneSecret = userOne.secretKey;
    config.userTwoSecret = userTwo.secretKey;
    config.voterSecret = userThree.secretKey;

    fs.writeFileSync(fileName, JSON.stringify(config, null, 2));
}

export function setDispute(id: anchor.BN) {
    console.log("Dispute ID #: ", id.toNumber().toString());

    const configContents = fs.readFileSync(fileName, 'utf8');

    const config: Config = JSON.parse(configContents);
    config.disputeId = id.toJSON();

    fs.writeFileSync(fileName, JSON.stringify(config, null, 2));
}

export function getMintInfo(): [Keypair, Keypair, number] {
  const configContents = fs.readFileSync(fileName, 'utf8');
  const config: Config = JSON.parse(configContents);

  let mintAuthArr = [];
  let repMintArr = [];
  for (const key in config.mintAuthSecret) {
    mintAuthArr.push(config.mintAuthSecret[key]);
  }

  for (const key in config.repMintSecret) {
    repMintArr.push(config.repMintSecret[key])
  }

  let mintAuthority = Keypair.fromSecretKey(Uint8Array.from(mintAuthArr));
  let repMint = Keypair.fromSecretKey(Uint8Array.from(repMintArr));
  let decimals = config.decimals;

  return [mintAuthority, repMint, decimals];
}

export function getProtocol(): Keypair {
  const configContents = fs.readFileSync(fileName, 'utf8');
  const config: Config = JSON.parse(configContents);

  let protocolArr = [];
  for (const key in config.protocolSecret) {
    protocolArr.push(config.protocolSecret[key]);
  }

  return Keypair.fromSecretKey(Uint8Array.from(protocolArr));
}

export function getDisputeID(): anchor.BN {
  const configContents = fs.readFileSync(fileName, 'utf8');
  const config: Config = JSON.parse(configContents);

  let id = new anchor.BN(config.disputeId);

  return id;
}

export function getUsers(): [Keypair, Keypair, Keypair] {
  const configContents = fs.readFileSync(fileName, 'utf8');
  const config: Config = JSON.parse(configContents);

  let userOneArr = [];
  let userTwoArr = [];
  let voterArr = [];
  for (const key in config.userOneSecret) {
    userOneArr.push(config.userOneSecret[key]);
  }

  for (const key in config.userTwoSecret) {
    userTwoArr.push(config.userTwoSecret[key]);
  }

  for (const key in config.voterSecret) {
    voterArr.push(config.voterSecret[key]);
  }

  let userOne = Keypair.fromSecretKey(Uint8Array.from(userOneArr));
  let userTwo = Keypair.fromSecretKey(Uint8Array.from(userTwoArr));
  let voter = Keypair.fromSecretKey(Uint8Array.from(voterArr));

  return [userOne, userTwo, voter];
}

export function getSingleUser(): Keypair {
  let users = getUsers();

  return users[user]; //CHANGE THIS AS NECESSARY FOR TESTING
}