import assert from 'node:assert/strict';
import { createPinia, setActivePinia } from 'pinia';
import { useTrackerStore, type LootItem } from '../src/stores/trackerStore';
import { priceLabel, unknownPrice, type PriceQuote, type PricingSnapshot } from '../src/utils/prices';
const known = (value: number): PriceQuote => ({ value, status:'known', source:'fixture', updated_at:'2026-10-20T08:00:00Z', received_at:'2026-10-20T08:00:00Z' });
const item = (id='123456', qty=2, valuation=unknownPrice()): LootItem => ({item_id:id, item_name:`Fixture ${id}`, qty, price:valuation.value ?? 0, timestamp:1, valuation, season_id:1601});
const snapshot = (prices: Record<string,PriceQuote>, revision=1, season=1601): PricingSnapshot => ({revision,season_id:season,source:'local_json',connection:'offline_local',last_success_at:null,error:null,available_count:Object.keys(prices).length,refresh_seconds:300,titrack_enabled:false,prices});
const freshStore = () => { setActivePinia(createPinia()); return useTrackerStore(); };
let checks=0;
function test(name:string, check:()=>void) {check();checks++;console.log(`PASS pricing: ${name}`);}
test('unknown loot survives positive threshold and history keeps quantity',()=>{
 const s=freshStore();s.minPriceThreshold=10;s.addDrop(item());
 assert.equal(s.sessionLootCount,2);assert.equal(s.sessionDrops[0].qty,2);assert.equal(s.currentMapDrops[0].qty,2);
 assert.equal(s.totalProfit,0);assert.equal(priceLabel(s.sessionDrops[0]),'—');assert.equal(s.passesLootFilter(s.sessionDrops[0]),false);
 s.minPriceThreshold=0;assert.equal(s.passesLootFilter(s.sessionDrops[0]),true);assert.equal(s.shouldAnimateDrop(s.latestDrop!),false);
 assert.equal(s.sessionLootCount,2);
});
test('repeated same ID never aliases map/session/latestDrop',()=>{
 const s=freshStore();s.addDrop(item('123456',2,known(3)));const latest=s.latestDrop;
 assert.notEqual(s.sessionDrops[0],s.currentMapDrops[0]);assert.notEqual(s.latestDrop,s.sessionDrops[0]);
 s.addDrop(item('123456',3,known(3)));assert.equal(s.sessionDrops[0].qty,5);assert.equal(s.currentMapDrops[0].qty,5);assert.equal(latest!.qty,2);
 assert.equal(s.sessionLootCount,5);assert.equal(s.totalProfit,15);assert.equal(s.currentMapProfit,15);
});
test('threshold changes affect visibility and only future effects',()=>{
 const s=freshStore();s.minPriceThreshold=10;s.addDrop(item('123456',1,known(2)));const latest=s.latestDrop;
 assert.equal(s.totalProfit,2);assert(!s.shouldAnimateDrop(latest!));assert(!s.passesLootFilter(s.sessionDrops[0]));
 s.minPriceThreshold=0;assert(s.passesLootFilter(s.sessionDrops[0]));assert.equal(s.latestDrop,latest);
 s.addDrop(item('123456',1,known(2)));assert(s.shouldAnimateDrop(s.latestDrop!));assert.equal(s.sessionLootCount,2);
});
test('new prices revalue both scopes without duplicate pickups or effects',()=>{
 const s=freshStore();s.addDrop(item('123456',3));s.addExpense({id:'cost',name:'cost',qty:1,total:2,timestamp:1});const latest=s.latestDrop;
 s.applyPricing(snapshot({'123456':known(4)}));assert.equal(s.totalProfit,10);assert.equal(s.currentMapProfit,10);assert.equal(s.sessionLootCount,3);assert.equal(s.latestDrop,latest);
 s.applyPricing(snapshot({'123456':known(4)},2));assert.equal(s.totalProfit,10);assert.equal(s.sessionDrops[0].qty,3);
 s.applyPricing(snapshot({'123456':known(2)},1));assert.equal(s.totalProfit,10); // older snapshot ignored
 s.applyPricing(snapshot({'123456':known(0)},3));assert.equal(s.totalProfit,-2);assert.equal(priceLabel(s.sessionDrops[0]),'0.0');
});
test('map transition retains session quantities and reprices independent scopes',()=>{
 const s=freshStore();s.addDrop(item('123456',2,known(3)));s.resetMap();s.addDrop(item('123456',1,known(3)));
 s.applyPricing(snapshot({'123456':known(5)}));assert.equal(s.sessionDrops[0].qty,3);assert.equal(s.currentMapDrops[0].qty,1);
 assert.equal(s.totalProfit,15);assert.equal(s.currentMapProfit,5);assert.equal(s.sessionLootCount,3);
});
test('tax weights are preserved across repricing, FE is exempt',()=>{
 const s=freshStore();s.applyTax=true;s.addDrop(item('123456',2));s.applyTax=false;s.addDrop(item('123456',2));
 s.applyPricing(snapshot({'123456':known(4)}));assert.equal(s.totalProfit,15);assert.equal(s.sessionLootCount,4);
 s.applyTax=true;s.addDrop(item('100300',2,known(1)));assert.equal(s.totalProfit,17);
});
test('stale and unknown differ from genuine zero; seasons never cross-reprice',()=>{
 const s=freshStore();s.addDrop(item('123456',2,known(3)));s.applyPricing(snapshot({'123456':known(99)},1,1501));assert.equal(s.totalProfit,6);
 s.applyPricing(snapshot({'123456':{...known(4),status:'stale'}},2));assert.equal(s.totalProfit,8);assert(priceLabel(s.sessionDrops[0]).includes('stale'));
 s.applyPricing(snapshot({},3));assert.equal(s.totalProfit,0);assert.equal(priceLabel(s.sessionDrops[0]),'—');assert.equal(s.sessionLootCount,2);
});
test('invalid loot rejected; inventory quantity retained on price update',()=>{
 const s=freshStore();s.addDrop(item('123456',-1));s.addDrop(item('../x',1));assert.equal(s.sessionLootCount,0);
 s.inventoryItems=[{item_id:'123456',item_name:'fixture',qty:3,price:0,valuation:unknownPrice(),season_id:1601}];
 s.applyPricing(snapshot({'123456':known(2)}));assert.equal(s.inventoryItems[0].qty,3);assert.equal(s.inventoryValue,6);
});
test('repricing removes rounded per-event residue without changing quantities',()=>{
 const s=freshStore();s.addDrop(item('123456',1,known(0.005)));s.addDrop(item('123456',1,known(0.005)));
 assert.equal(s.totalProfit,0.02);s.applyPricing(snapshot({'123456':known(0)}));assert.equal(s.totalProfit,0);assert.equal(s.currentMapProfit,0);assert.equal(s.sessionLootCount,2);
});
console.log(`PASS ${checks} pricing store scenarios`);
