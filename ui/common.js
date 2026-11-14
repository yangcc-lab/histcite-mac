'use strict';
window.H = {
 sourceId: r => r.fields?.UT?.join(" ") || r.id,
 $: id => document.getElementById(id),
 el: (tag, text, cls) => { const e=document.createElement(tag); if(text!==undefined)e.textContent=text; if(cls)e.className=cls; return e; },
 async invoke(name,args={}) {
  if(window.__HISTCITE_EXPORT__) {
   const data=window.__HISTCITE_EXPORT__;
   if(['current_analysis','initial_analysis'].includes(name))return data;
   if(name==='mutate_collection'&&['mark','unmark'].includes(args.operation?.action)){const op=args.operation;data.collection.marks=op.action==='mark'?[...new Set([...data.collection.marks,...op.ids])]:data.collection.marks.filter(id=>!op.ids.includes(id));data.collection.revision++;return data;}
   if(name==='open_record'){if(!H.canOpen(data,args.index))throw new Error('This record detail is omitted by the HTML presentation display limits.');window.open(`detail.html?id=${encodeURIComponent(data.records[args.index].id)}&index=${args.index}`);return;}
   if(name==='open_list'){window.open('index.html?subset='+encodeURIComponent(JSON.stringify(args.ids))+'&title='+encodeURIComponent(args.title));return;}
   if(name==='open_graph'){window.open('index.html?window=graph'+(args.graphId?'&graph='+encodeURIComponent(args.graphId):''));return;}
   if(name==='print_window'){window.print();return;}
   if(name==='open_external'){window.open(args.url,'_blank','noopener');return;}
   if(name==='save_export'&&args.format==='json')args.content=JSON.stringify(data,null,2);
   if(name==='save_export'&&args.format==='hci'){const selected=new Set(args.ids||data.records.map(r=>r.id));const c=data.collection;args.content='FN Thomson Reuters Web of Knowledge\nVR 1.0\nMG HistCite Mac 0.3\n'+Object.entries({MT:c.title,MC:c.description,MO:c.created,MN:c.comment}).map(([k,v])=>k+' '+String(v||'').replaceAll('\n','\n   ')+'\n').join('')+Object.entries(c.tags||{}).map(([k,t],i)=>(i?'   ':'TD ')+k+': '+t.description.replace(/[\r\n]/g,' ')+'\n').join('')+'ER\n\n'+data.records.filter(r=>selected.has(r.id)).map(r=>{const f=structuredClone(r.fields);delete f.TG;const tags=Object.entries(c.tags||{}).filter(([,t])=>t.records.includes(r.id)).map(([k])=>k);if(tags.length)f.TG=[tags.join(' ')];return H.wosRecord(f);}).join('\n')+'EF\n';}
   if(name==='save_export' && args.content){const bytes=args.format==='png'?Uint8Array.from(atob(args.content),c=>c.charCodeAt(0)):args.content;const url=URL.createObjectURL(new Blob([bytes]));const a=H.el('a');a.href=url;a.download='histcite-analysis.'+args.format;a.click();setTimeout(()=>URL.revokeObjectURL(url),1000);return;}
   throw new Error('HTML presentation is read-only. Open the collection in HistCite Mac to edit it.');
  }
  if(window.__TAURI__?.core?.invoke)return window.__TAURI__.core.invoke(name,args);
  throw new Error('請使用 HistCite Mac 應用程式。');
 },
 canOpen:(d,i)=>!window.__HISTCITE_EXPORT__||!d.settings?.html_limits||d.lcs[i]>=d.settings.html_lcs_min||(d.records[i].gcs??0)>=d.settings.html_gcs_min,
 wosRecord:f=>{let text='PT '+(f.PT?.[0]||'J')+'\n';for(const[key,values]of Object.entries(f)){if(key==='PT')continue;values.forEach((v,i)=>text+=(i?'   ':key+' ')+v+'\n');}return text+'ER\n';},
 async wosLookup(d,reference){const parts=reference.split(',').map(x=>x.trim()),doi=reference.match(/10\.\d{4,9}\/[^\s,;\]]+/i)?.[0]||'',values={reference,author:parts[0],year:parts[1]||'',source:parts[2]||'',doi};let template=d.settings?.wos_link_template||'https://www.webofscience.com/';if(!/\{(reference|author|year|source|doi)\}/.test(template)){if(!await H.form('Web of Science lookup',[{name:'reference',label:'Reference to copy into Cited Reference Search',type:'textarea',value:reference,full:true}],{description:'Opens Web of Science in your browser. Configure a search URL template in Settings if your institution provides one.',submit:'Open Web of Science'}))return;}const url=template.replace(/\{(reference|author|year|source|doi)\}/g,(_,key)=>encodeURIComponent(values[key]));return H.invoke('open_external',{url});},
 dateParts:r=>{const p=(r.publication_date||'').toUpperCase(),months=['JAN','FEB','MAR','APR','MAY','JUN','JUL','AUG','SEP','OCT','NOV','DEC'],m=months.findIndex(x=>p.startsWith(x)),seasons={WIN:1,SPR:4,SUM:7,FAL:10,AUT:10};return [m>=0?m+1:seasons[p.slice(0,3)]||0,Number(p.split(/\s+/)[1]?.match(/^\d{1,2}$/)?.[0]||0)];},
 dateSort:(a,b)=>{const x=H.dateParts(a),y=H.dateParts(b);return (a.year??9999)-(b.year??9999)||x[0]-y[0]||x[1]-y[1]||a.journal.localeCompare(b.journal)||(parseFloat(a.volume)||0)-(parseFloat(b.volume)||0)||(parseFloat(a.issue)||0)-(parseFloat(b.issue)||0)||(parseFloat(a.page)||0)-(parseFloat(b.page)||0)||(a.article_number&&b.article_number?a.article_number.localeCompare(b.article_number):0)||a.id.localeCompare(b.id);},
 order:data=>data.records.map((_,i)=>i).sort((a,b)=>H.dateSort(data.records[a],data.records[b])),
 author:s=>s.replace(/,\s*/g,' '),
 date:r=>`${r.year??''} ${(r.publication_date||'').replace(new RegExp('^'+r.year+'\\s*'),'')}`.trim(),
 source:r=>`${r.journal}. ${H.date(r)}; ${r.volume}${r.issue?' ('+r.issue+')':''}${r.page?': '+r.page+(r.end_page&&r.end_page!==r.page?'-'+r.end_page:''):r.article_number?': '+r.article_number:''}`,
 error:e=>{const box=document.getElementById('error');box.hidden=false;box.querySelector('span').textContent=String(e?.message||e);},
 metric:(d,i,key)=>key==='lcs'||key==='lcr'?d[key][i]:key==='gcs'?d.records[i].gcs:key==='cr'?d.records[i].references.length:key==='year'?d.records[i].year:d.metrics?.[i]?.[key],
 display:(value,key)=>value===null||value===undefined||(['lcsb','lcse'].includes(key)&&value<0)?'—':['lcst','gcst'].includes(key)?Number(value).toFixed(2):String(value),
 metricHints:{
  lcs:'Local Citation Score — citations received from records in this collection.',
  lcsx:'Local Citation Score excluding self-citations — citing and cited records have no author in common.',
  gcs:'Global Citation Score — the citation count supplied by Web of Science at export time.',
  ocs:'Other Citation Score — the count supplied in the TS field.',
  lcr:'Local Cited References — references that match records in this collection.',
  cr:'Cited References — the number of entries in the reference list.',
  na:'Number of Authors.',
  lcst:'Local Citation Score per year — LCS divided by inclusive years from publication to the collection end.',
  gcst:'Global Citation Score per year — GCS divided by inclusive years from publication to the collection end.',
  lcsb:'Local Citation Score in the beginning — citations in the beginning period selected in Settings.',
  lcse:'Local Citation Score in the end — citations in the ending period selected in Settings.',
  ratio:'Ratio LCSe/LCSb — ending-period citations divided by beginning-period citations.'
 },
 metricHint:(key,total=false)=>(total?'Total across the records in this group. ':'')+(H.metricHints[key]||''),
 hint:(element,text)=>{if(text){element.setAttribute('title',text);element.setAttribute('aria-description',text);}return element;},
 recordHint:(r,number)=>`${number?number+' · ':''}${r.authors.map(H.author).join('; ')}\n${r.title}\n${H.source(r)}\n${r.document_type||'Unknown document type'}`,
 groupHint:g=>`${g.occurrences??g.indices.length} records${g.lcs===undefined?'':', TLCS '+g.lcs}${g.gcs==null?'':', TGCS '+g.gcs}${g.description?'\n'+g.description:''}`,
 csv:rows=>'\ufeff'+rows.map(row=>row.map(value=>{if(typeof value==='number')return value;let s=String(value??'');if(/^[=+\-@\t\r]/.test(s))s="'"+s;return '"'+s.replace(/"/g,'""')+'"';}).join(',')).join('\r\n')+'\r\n',
 hIndex:values=>values.filter(v=>v!==null&&v!==undefined).sort((a,b)=>b-a).filter((n,i)=>n>=i+1).length,
 button:(label,action,cls='link')=>{const b=H.el('button',label,cls);b.type='button';b.onclick=action;return b;},
 form(title,fields,{submit='Apply changes',description='',wide=false}={}) {
  return new Promise(resolve=>{
   const dialog=H.el('dialog',undefined,wide?'wide':'');dialog.setAttribute('aria-label',title);const form=H.el('form');form.method='dialog';form.append(H.el('h2',title));if(description)form.append(H.el('p',description));const grid=H.el('div',undefined,'form-grid');
   for(const f of fields){const label=H.el('label',f.label||f.name);let input;if(f.type==='select'){input=H.el('select');for(const o of f.options){const [v,t]=Array.isArray(o)?o:[o,o];const option=H.el('option',t);option.value=v;input.append(option);}}else if(f.type==='textarea'){input=H.el('textarea');input.rows=f.rows||4;}else{input=H.el('input');input.type=f.type||'text';}input.name=f.name;input.id='field-'+f.name;if(f.type==='checkbox'){input.checked=!!f.value;label.prepend(input);}else{input.value=f.value??'';label.append(input);}if(f.required)input.required=true;if(f.min!==undefined)input.min=f.min;if(f.max!==undefined)input.max=f.max;if(f.step)input.step=f.step;if(f.full)label.classList.add('full');grid.append(label);}
   const actions=H.el('div',undefined,'dialog-actions'),ok=H.el('button',submit),cancel=H.el('button','Cancel');ok.type='submit';cancel.type='button';cancel.onclick=()=>dialog.close();actions.append(ok,cancel);form.append(grid,actions);dialog.append(form);document.body.append(dialog);
   let result=null;form.onsubmit=e=>{e.preventDefault();result={};for(const f of fields){const input=form.elements.namedItem(f.name);result[f.name]=f.type==='checkbox'?input.checked:f.type==='number'?Number(input.value):input.value;}dialog.close();};dialog.addEventListener('close',()=>{dialog.remove();resolve(result);},{once:true});dialog.showModal();
  });
 },
 async confirm(title,text,submit='Continue'){return !!await H.form(title,[],{description:text,submit});},
 async editRecord(data,id=null,reference='') {
  const r=data.records.find(r=>r.id===id),f=structuredClone(r?.fields||{});if(!r){f.PT=['J'];f.UT=['manual:'+crypto.randomUUID()];f.NR=['0'];const parts=reference.split(',').map(x=>x.trim());if(reference){f.AU=[parts[0].replace(/\s+(\S+)$/,', $1')];if(/^\d{4}$/.test(parts[1]))f.PY=[parts[1]];f.SO=[parts[2]||''];for(const part of parts.slice(3)){if(/^V\S/.test(part))f.VL=[part.slice(1)];if(/^P\S/.test(part))f.BP=[part.slice(1)];if(/^DOI /i.test(part))f.DI=[part.slice(4)];}}}
  const labels={UT:'WoS ID / unique ID',AU:'Author(s), one per line',AF:'Full author names',TI:'Title',SO:'Source',J9:'Source abbreviation',PY:'Year',PD:'Month / day',VL:'Volume',IS:'Issue',BP:'Start page',EP:'End page',AR:'Article number',PT:'Publication type',DT:'Document type',LA:'Language',DI:'DOI',TC:'Global citation score (GCS)',TS:'Other citation score (OCS)',NR:'Declared reference count',C1:'Addresses, one per line',RP:'Reprint address',EM:'E-mail',DE:'Author keywords',ID:'KeyWords Plus',NB:'Comment',AB:'Abstract',CR:'Cited references, one per line'};
  const areas=['AU','AF','C1','RP','CR','AB'];const fields=Object.entries(labels).map(([name,label])=>({name,label,type:areas.includes(name)?'textarea':'text',value:(f[name]||[]).join(areas.includes(name)?'\n':' '),required:['UT','TI'].includes(name),full:['TI','SO','C1','CR','AB'].includes(name),rows:name==='CR'?8:name==='AB'?5:3}));
  const result=await H.form(r?'Edit record':'New record',fields,{wide:true,description:'Changes are saved to the working collection. Source files stay intact. Use File → Save to save a portable collection.'});if(!result)return null;
  for(const [key,value]of Object.entries(result)){if(String(value).trim())f[key]=String(value).split('\n');else delete f[key];}
  let text='PT '+(f.PT?.[0]||'J')+'\n';for(const[key,values]of Object.entries(f)){if(key==='PT')continue;values.forEach((v,i)=>text+=(i?'   ':key+' ')+v+'\n');}text+='ER\n';
  return H.invoke('mutate_collection',{revision:data.collection?.revision||0,operation:{action:'record',id,text}});
 },
 listen:async(name,callback)=>{
  try{return await window.__TAURI__?.event?.listen?.(name,callback);}
  catch(e){H.error('Window notifications unavailable: '+String(e?.message||e));}
 },
 onChange(callback,revision=()=>null){
  if(!window.__TAURI__||window.__HISTCITE_EXPORT__)return;
  let running=false,pending=false,check=false,disposed=false;
  const refresh=async(force=false)=>{
   if(disposed)return;
   if(force)pending=true;else check=true;
   if(running)return;
   running=true;
   try{while(!disposed&&(pending||check)){
    const reload=pending;pending=false;check=false;
    if(reload||await H.invoke('current_revision')!==revision())await callback();
   }}catch(e){H.error(e);}finally{running=false;}
  };
  const focus=()=>refresh(),visible=()=>{if(document.visibilityState==='visible')refresh();};
  window.addEventListener('focus',focus);
  document.addEventListener('visibilitychange',visible);
  const listener=H.listen('collection-changed',()=>refresh(true));
  window.addEventListener('pagehide',()=>{
   disposed=true;window.removeEventListener('focus',focus);document.removeEventListener('visibilitychange',visible);
   listener.then(unlisten=>unlisten?.()).catch(H.error);
  },{once:true});
 },
};
if(window.__HISTCITE_EXPORT__)document.documentElement.classList.add('presentation');

// Original menus open on hover and allow 500 ms to cross into the submenu.
// Keep click/touch activation and keyboard navigation on the same open state.
H.initMenus = () => {
 const menus=[...document.querySelectorAll('.menu')];
 let timer=null,keyboardMode=false;
 const stopTimer=()=>{clearTimeout(timer);timer=null;};
 const setOpen=(menu,open)=>{menu.open=open;menu.querySelector('summary').setAttribute('aria-expanded',String(open));};
 const closeAll=()=>{stopTimer();for(const menu of menus)setOpen(menu,false);};
 const open=menu=>{stopTimer();for(const other of menus)setOpen(other,other===menu);};
 const items=menu=>[...menu.querySelectorAll('button:not(:disabled),select:not(:disabled),input:not(:disabled),a[href]')].filter(e=>e.getClientRects().length);
 for(const [i,menu] of menus.entries()){
  const summary=menu.querySelector('summary'),panel=menu.querySelector(':scope > div');
  panel.id||='menu-panel-'+i;summary.setAttribute('aria-controls',panel.id);setOpen(menu,false);
  menu.addEventListener('pointerenter',e=>{if(e.pointerType==='touch')return;keyboardMode=false;open(menu);});
  menu.addEventListener('pointerleave',e=>{if(e.pointerType==='touch')return;stopTimer();timer=setTimeout(()=>{if(!keyboardMode||!menu.contains(document.activeElement))setOpen(menu,false);},500);});
  // Hover has already opened the menu before a mouse click reaches summary.
  summary.addEventListener('click',e=>{e.preventDefault();open(menu);});
  menu.addEventListener('click',e=>{if(e.target.closest('button:not(:disabled),a[href]')){closeAll();if(menu.contains(document.activeElement))summary.focus({preventScroll:true});}});
  // During pointer focus changes activeElement can temporarily be body.
  // Closing at that point removes the button before its click is delivered.
  menu.addEventListener('focusout',e=>{if(e.relatedTarget&&!menu.contains(e.relatedTarget))setOpen(menu,false);});
  menu.addEventListener('keydown',e=>{
   keyboardMode=true;
   if(e.key==='Escape'){e.preventDefault();e.stopPropagation();closeAll();summary.focus();return;}
   // Native form controls retain their own arrow-key behavior.
   if(e.target.matches('select,input,textarea'))return;
   const list=()=>items(menu),current=()=>list().indexOf(document.activeElement);
   if(e.key==='ArrowDown'||e.key==='ArrowUp'){
    e.preventDefault();open(menu);const all=list(),pos=current(),step=e.key==='ArrowDown'?1:-1;
    all[(pos<0?(step>0?0:all.length-1):(pos+step+all.length)%all.length)]?.focus();
   }else if(e.key==='ArrowRight'||e.key==='ArrowLeft'){
    e.preventDefault();const next=menus[(i+(e.key==='ArrowRight'?1:-1)+menus.length)%menus.length],inPanel=e.target!==summary;
    open(next);(inPanel?items(next)[0]:next.querySelector('summary'))?.focus();
   }else if(e.key==='Home'||e.key==='End'){
    e.preventDefault();open(menu);const all=list();(e.key==='Home'?all[0]:all.at(-1))?.focus();
   }else if((e.key==='Enter'||e.key===' ')&&e.target===summary){
    e.preventDefault();open(menu);list()[0]?.focus();
   }
  });
 }
 document.addEventListener('pointerdown',e=>{if(!e.target.closest('.menu'))closeAll();});
 document.addEventListener('keydown',e=>{if(e.key==='Escape')closeAll();});
 window.addEventListener('blur',closeAll);
 return {closeAll};
};
