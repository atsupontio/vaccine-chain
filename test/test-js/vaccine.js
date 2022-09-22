const { ApiPromise, WsProvider, Keyring } = require('@polkadot/api');
const BN = require('bn.js');

const main = async() => {
  const provider = new WsProvider('ws://127.0.0.1:9944');
  //const provider = new HttpProvider('http://localhost:9933');
  const api = await ApiPromise.create({ provider });
  const PHRASE_TEST_ACCOUNT = 'speak sentence monster because comfort feature puppy area team piece plug field';

  const PHRASE_ROOT = 'fish dash budget stairs hire reason mention forest census copper kid away';
  const keyring = new Keyring({ type: 'sr25519' });

  // TEST ACCOUNT
  const TEST_ACCOUNT = keyring.addFromUri("//Alice");

  const Role = {
    SYSMAN:0,
    VM:1,
    VAO:2,
    VAD:3,
    USER:4,
  }

  const Vac = {
    COVID19:0,
    FLU:1,
    HPV:2,
    RUBELLA:3,
  }



  const unsub = await api.tx.vaccine
  .registerVacInfo('84151fda-d9e3-4f6d-9ed5-4f1eb4709e8f','1',Vac.COVID19)
  .signAndSend(TEST_ACCOUNT, (result) => {
    console.log(`Current status is ${result.status}`);

  if (result.status.isFinalized) {
      console.log(`Transaction finalized at blockHash ${result.status.asFinalized}`);
      console.log("Create campaign successfully");
      unsub();
    }
  });

}




main();