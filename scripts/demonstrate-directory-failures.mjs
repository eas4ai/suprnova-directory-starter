import { snapshot } from './verification-workspace.mjs';
import {readFileSync,writeFileSync,rmSync} from 'node:fs';
import {spawnSync} from 'node:child_process';
import {join, dirname, resolve} from 'node:path';
import {fileURLToPath} from 'node:url';
import {randomBytes} from 'node:crypto';
import assert from 'node:assert/strict';
const source=resolve(process.argv[2] ?? join(dirname(fileURLToPath(import.meta.url)), '..'));
// Mutate only disposable copies; unchanged integration assertions must fail at runtime.
const cases=[
['DIR-001','queries.rs','.filter(listing::Column::OwnerId.eq(actor_id))','.filter(listing::Column::OwnerId.is_not_null())'],
['DIR-002','queries.rs','unsafe_html: false','unsafe_html: true'],
['DIR-003','workflow.rs','.filter(revision::Column::Status.eq("submitted"))','.filter(revision::Column::Status.is_in(["submitted", "draft"]))'],
['DIR-004','queries.rs','let id = listing.approved_revision_id.ok_or_else(missing)?;','let id = listing.current_revision_id.ok_or_else(missing)?;'],
['DIR-005','queries.rs',"e.mode IN ('free', 'live')","e.mode IN ('free', 'live', 'test')"],
['DIR-006','queries.rs','page.size > 100','page.size > 1000'],
['DIR-007','queries.rs','listing::Column::Suspended.eq(false)','listing::Column::Suspended.is_not_null()'],
['DIR-008','queries.rs','"checkout"','"none"'],
];
for(const [id,file,before,after] of cases){
 const work=snapshot(source,'directory-negative-');
 try {
 const path=join(work,'src/listings',file),original=readFileSync(path,'utf8');
 assert.ok(original.includes(before));writeFileSync(path,original.replace(before,after));
 const env=Object.fromEntries(['PATH','HOME','CARGO_HOME','RUSTUP_HOME'].filter(k=>process.env[k]).map(k=>[k,process.env[k]]));
 Object.assign(env,{APP_ENV:'test',APP_DEBUG:'false',APP_URL:'http://directory.test',APP_KEY:randomBytes(32).toString('base64url'),MAIL_FROM:'test@example.test',DATABASE_URL:`sqlite://${join(work,'directory.db')}`,DB_LOGGING:'false',SESSION_SECURE:'false',DIRECTORY_MEDIA_ROOT:join(work,'storage/private/directory'),TMPDIR:work,CARGO_BUILD_JOBS:'2',CARGO_PROFILE_DEV_DEBUG:'0',CARGO_INCREMENTAL:'0',CARGO_TARGET_DIR:join(source,'target')});
 const result=spawnSync('cargo',['test','--locked','--test','directory_workflows','--','--nocapture'],{cwd:work,env,encoding:'utf8',timeout:600000,detached:true,maxBuffer:20*1024*1024});
 if (result.pid) { try { process.kill(-result.pid, 'SIGKILL'); } catch(error) { if(error.code !== 'ESRCH') throw error; } }
 if(result.error) throw result.error;
 const output=result.stdout+result.stderr;
 assert.equal(result.status,101,`${id}: expected runtime test failure, got ${result.status}`);
 assert.match(output,/test directory_workflows_contract \.\.\. FAILED/);
 assert.match(output,/panicked at tests\/directory_workflows.rs/);
 const failure=output.indexOf('panicked at tests/directory_workflows.rs');
 console.log(id+' rejected: '+output.slice(failure,failure+320));
 } finally {rmSync(work,{recursive:true,force:true});}
}
