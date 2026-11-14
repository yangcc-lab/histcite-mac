'use strict';
// HistCite's DOT conventions, backed by original Graph Maker exports in compatibility/.
// Preserve the legacy hash capacity and implicit endpoint insertion order.
// Callers keep the small ordering state with saved graphs for repeatable layouts.
const GraphLayout = (() => {
 const number=n=>Number(n.toPrecision(15)).toString();
 const quote=v=>'"'+String(v).replaceAll('\\','\\\\').replaceAll('"','\\"').replace(/[\r\n]/g,' ')+'"';
 const attributes=o=>' ['+Object.keys(o).sort().map(k=>k+'='+quote(typeof o[k]==='number'?number(o[k]):o[k])).join(', ')+']';
 const months=['','JAN','FEB','MAR','APR','MAY','JUN','JUL','AUG','SEP','OCT','NOV','DEC'];
 function hash(value){let h=0;for(const c of String(value)){h=(h+c.charCodeAt(0))>>>0;h=(h+(h<<10))>>>0;h^=h>>>6;}h=(h+(h<<3))>>>0;h^=h>>>11;return(h+(h<<15))>>>0;}
 // Perl 5.12: prepend collisions; split only after a collision at capacity,
 // reversing chains moved to the upper half. Clearing a hash keeps its capacity.
 function hashOrder(keys,capacity=8) {
  let size=8;while(size<capacity&&size<1048576)size*=2;
  let buckets=Array.from({length:size},()=>[]),count=0;
  for(const value of keys){const key=String(value),at=hash(key)&(buckets.length-1),bucket=buckets[at];
   if(bucket.includes(key))continue;
   const collision=bucket.length;bucket.unshift(key);count++;
   if(collision&&count>buckets.length-1){const old=buckets.length;buckets=buckets.concat(Array.from({length:old},()=>[]));
    for(let i=0;i<old;i++){const keep=[];for(const k of buckets[i]){if(hash(k)&old)buckets[i+old].unshift(k);else keep.push(k);}buckets[i]=keep;}
   }
  }
  return {keys:buckets.flat(),capacity:buckets.length};
 }
 function buildGraph(data,numbers,selected,edges,s,dimensions={width:900,height:600},ordering={nodeBuckets:8,rankBuckets:8}) {
  const score=i=>s.score==='gcs'?(data.records[i].gcs||0):data.lcs[i];
  const node=i=>'n'+(numbers.get(i)-1),max=selected.reduce((n,i)=>Math.max(n,score(i)),1);
  const byNumber=new Map(selected.map(i=>[String(numbers.get(i)-1),i]));
  const mapOrder=hashOrder(byNumber.keys(),ordering.nodeBuckets),inserted=new Set(),outgoing=new Map();
  if(s.links)for(const e of edges){if(!outgoing.has(e.cited))outgoing.set(e.cited,[]);outgoing.get(e.cited).push(e.citing);}
  for(const list of outgoing.values())list.sort((a,b)=>numbers.get(a)-numbers.get(b));
  // GraphViz.pm creates missing endpoints immediately when adding an edge.
  for(const key of mapOrder.keys){const i=byNumber.get(key);inserted.add(i);for(const target of outgoing.get(i)||[])inserted.add(target);}
  const ordered=[...inserted];ordering.nodeBuckets=mapOrder.capacity;
  const groups=new Map(),years=new Set();
  const month=i=>s.months?months[H.dateParts(data.records[i])[0]]:'';
  const rank=i=>String(data.records[i].year)+(month(i)||'');
  for(const i of ordered){const r=rank(i);if(!groups.has(r))groups.set(r,[]);groups.get(r).push(i);years.add(data.records[i].year);}
  if(s.gaps){const first=[...years].reduce((a,b)=>Math.min(a,b),Infinity),last=[...years].reduce((a,b)=>Math.max(a,b),-Infinity);for(let y=first;y<=last;y++)years.add(y);}
  const layers=[];for(const y of [...years].sort((a,b)=>a-b)){layers.push(String(y));if(s.months)for(const m of months.slice(1)){if(groups.has(y+m))layers.push(y+m);}}
  let dot='digraph test {\n';
  if(s.size==='window')dot+=`\tsize=${quote(number(Math.max(100,dimensions.width-20)/96)+','+number(Math.max(100,dimensions.height-25)/96))};\n`;
  if(s.size==='letter')dot+='\tsize="6.5,9";\n';
  dot+='\tratio="fill";\n';if(s.merge)dot+='\tconcentrate=true;\n';
  // The legacy export ignores its Node distance controls. Preserve that in
  // compatible layout; explicit custom spacing is a separately labelled option.
  if(s.spacing==='custom')dot+=`\tnodesep=${quote(s.x)};\n\tranksep=${quote(s.y+(s.equalSpacing?' equally':''))};\n`;
  dot+='\tnode'+attributes({fixedsize:'true',fontsize:s.nodeFont,shape:s.shape==='none'?'plaintext':s.shape})+';\n';
  dot+='\tedge'+attributes({arrowhead:s.direction==='forward'?s.arrow:'none',arrowsize:s.arrowSize,arrowtail:s.direction==='backward'?s.arrow:'none'})+';\n';
  for(const layer of layers){const isMonth=layer.length>4,font=isMonth?s.monthFont:s.yearFont,y=Number(layer.slice(0,4));
   const label=isMonth?layer.slice(4):layer+(s.yearcount?' ('+(data.year_counts?.[y]||0)+')':'');
   dot+='\ty'+layer+attributes({fontsize:font,height:(font+2)*.0139,label,margin:0,rank:layer,shape:'plaintext',width:(label.length*font/2+1.5)*.0185185066666667})+';\n';}
  for(const i of ordered){const value=score(i),diameter=s.scaled&&s.shape!=='none'?(value?Math.sqrt(value/max)*.75:.1)*s.scaleFactor:s.fixed;
   dot+='\t'+node(i)+attributes({URL:String(numbers.get(i)-1),height:diameter,label:s.id==='inside'?String(numbers.get(i)):'',rank:rank(i),style:s.scaled&&!value&&s.shape!=='none'?'filled':'',width:diameter})+';\n';}
  const lines=[];
  if(s.links)for(const e of edges)lines.push({from:node(e.cited),to:node(e.citing),attrs:''});
  if(s.id==='outside')for(const i of ordered)lines.push({from:node(i),to:node(i),attrs:attributes({arrowhead:'none',color:'white',headlabel:String(numbers.get(i)),labeldistance:s.proximity,labelfontsize:s.nodeFont})});
  for(let i=1;i<layers.length;i++)lines.push({from:'y'+layers[i-1],to:'y'+layers[i],attrs:attributes({arrowhead:'normal',arrowtail:'none',color:'white',style:'invis'})});
  lines.sort((a,b)=>a.from<b.from?-1:a.from>b.from?1:a.to<b.to?-1:a.to>b.to?1:0);
  for(const l of lines)dot+=`\t${l.from} -> ${l.to}${l.attrs};\n`;
  // The original writes PS first, then the on-screen graph with the same
  // lexical rank hash. A growth can change tie order between these two passes.
  const psRanks=hashOrder(layers,ordering.rankBuckets),imageRanks=hashOrder(layers,psRanks.capacity);
  ordering.rankBuckets=imageRanks.capacity;
  const finish=ranks=>dot+ranks.keys.map(layer=>`\t{rank=same; y${layer}${(groups.get(layer)||[]).map(i=>'; '+node(i)).join('')}}\n`).join('')+'}\n';
  return {dot:finish(imageRanks),psDot:finish(psRanks)};
 }
 const build=(...args)=>buildGraph(...args).dot;
 let active=null;
 function cancel(){active?.();active=null;}
 function layout(dot,formats=['svg']) {
  cancel();
  return new Promise((resolve,reject)=>{
   const source=`let _scriptName='';const HistCiteDot=${HistCiteDot.toString()};self.onmessage=async({data})=>{try{const out={};for(const format of data.formats){const m=await HistCiteDot({print:()=>{},printErr:()=>{}});m.FS.writeFile('/graph.dot',data.dot);m.callMain(['-q','-T'+format,'-o','/result','/graph.dot']);out[format]=m.FS.readFile('/result',{encoding:'utf8'});}self.postMessage({out});}catch(e){self.postMessage({error:String(e)});}};`;
   const url=URL.createObjectURL(new Blob([source],{type:'text/javascript'})),worker=new Worker(url);URL.revokeObjectURL(url);
   const finish=()=>{clearTimeout(timeout);worker.terminate();if(active===abort)active=null;};
   const abort=()=>{finish();reject(new DOMException('Layout superseded','AbortError'));};active=abort;
   const timeout=setTimeout(()=>{finish();reject(new Error('Graph layout exceeded 60 seconds. Reduce the selected record count and try again.'));},60000);
   worker.onmessage=({data})=>{finish();if(data.error)reject(new Error(data.error));else resolve(data.out);};
   worker.onerror=e=>{finish();reject(new Error(e.message||'Unable to start the graph engine.'));};
   worker.postMessage({dot,formats});
  });
 }
 return {build,buildGraph,layout,cancel};
})();
if(typeof module==='object')module.exports=GraphLayout;
