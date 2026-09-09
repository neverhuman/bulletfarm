//! Public private-render CLI controls using real local FFmpeg/agg derivatives.

use std::process::Command;

const FIXTURE: &str = r#"
import hashlib,json,os,runpy,shutil,signal,struct,subprocess,sys,time,zlib
from pathlib import Path
hub,root,case=Path(sys.argv[1]),Path(sys.argv[2]),sys.argv[3]
os.umask(0o077)
python=Path(sys.executable).resolve()
ffmpeg=Path(os.environ.get('DEMO_GIF_TEST_FFMPEG','/usr/bin/ffmpeg')).resolve()
agg=Path(os.environ.get('DEMO_GIF_TEST_AGG',str(Path.home()/'.cache/bullet-demo-gif/bin/agg'))).resolve()
implementation=hub/'scripts/lib/demo-gif-render.py'
def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def write(path,data):
 path.parent.mkdir(mode=0o700,parents=True,exist_ok=True)
 with path.open('xb') as out:out.write(data)
def js(path,value):write(path,(json.dumps(value)+'\n').encode())
def png(width,height,data):
 def chunk(name,data):return struct.pack('>I',len(data))+name+data+struct.pack('>I',zlib.crc32(name+data))
 scan=b''.join(b'\0'+data[y*width*3:(y+1)*width*3] for y in range(height))
 return b'\x89PNG\r\n\x1a\n'+chunk(b'IHDR',struct.pack('>IIBBBBB',width,height,8,2,0,0,0))+chunk(b'IDAT',zlib.compress(scan))+chunk(b'IEND',b'')
def portal(name,colorful=False,elapsed=None):
 path=root/name;path.mkdir(mode=0o700);js(path/'started.json',{'capture_status':'INCOMPLETE'})
 frames=[]
 for frame in range(len(elapsed) if elapsed is not None else 2):
  pixels=bytes(v for y in range(16) for x in range(32) for v in
   ((x*8,y*16,(x+y+frame)*5%256) if colorful else ((255,0,0) if (x+y+frame)%2 else (0,0,255))))
  file=f'frames/{frame}.png';write(path/file,png(32,16,pixels))
  frames.append({'file':file,'elapsed_ms':elapsed[frame] if elapsed is not None else 100.123+frame*123.456,'sha256':sha(path/file),
   'bytes':(path/file).stat().st_size,'width':32,'height':16,'source':'BROWSER_PNG_SCREENSHOT'})
 js(path/'observation.json',{'schema_version':'bullet.portal-capture.v1','capture_status':'CAPTURED',
  'bullet_live_admission':False,'product_completion':'UNVERIFIED','frames':frames})
 return path
def invoke(mode,kind,source,dest,extra=(),expect=0,reason=None,digest=None,executable=None,tool_hash=None,impl_hash=None):
 metadata=source/('manifest.json' if mode=='check' else 'observation.json' if kind=='portal' else 'session.cast.result.json')
 binary=executable or ffmpeg
 args=['bash',str(hub/f'scripts/demo-gif-{mode}.sh'),str(python),sha(python),impl_hash or sha(implementation),
  '--kind',kind,'--input',str(source),'--output',str(dest),'--input-sha256',digest or sha(metadata),
  '--ffmpeg',str(binary),'--ffmpeg-sha256',tool_hash or sha(binary)]
 if kind=='terminal':args+=['--agg',str(agg),'--agg-sha256',sha(agg)]
 result=subprocess.run(args+list(extra),capture_output=True,text=True,timeout=30)
 print(json.dumps({'mode':mode,'kind':kind,'expected':expect,'exit':result.returncode,'stderr':result.stderr}),flush=True)
 assert result.returncode==expect,(result.stdout,result.stderr)
 if reason:assert reason in result.stderr,(reason,result.stderr)
 if expect!=0:assert not (dest/'manifest.json').exists()
 return result
def observation(path):
 result=subprocess.run([str(ffmpeg),'-nostdin','-v','error','-copyts','-i',str(path),'-fps_mode','passthrough',
  '-c:v','rawvideo','-pix_fmt','rgb24','-enc_time_base','-1','-f','framehash','-hash','sha256','-'],
  check=True,capture_output=True,text=True,timeout=10).stdout.splitlines()
 tb=next(line[7:] for line in result if line.startswith('#tb 0: '))
 rows=[[part.strip() for part in line.split(',')] for line in result if not line.startswith('#')]
 return [{'pts':int(row[2]),'timebase':tb} for row in rows],[row[5] for row in rows]
def gce_offsets(data):
 cursor=13+(3*2**((data[10]&7)+1) if data[10]&128 else 0);found=[]
 while data[cursor]!=0x3b:
  tag=data[cursor];cursor+=1
  if tag==0x21:
   label=data[cursor];cursor+=1
   if label==0xf9:found.append(cursor-2)
  else:
   assert tag==0x2c;packed=data[cursor+8];cursor+=9+(3*2**((packed&7)+1) if packed&128 else 0)
   cursor+=1
  while data[cursor]:cursor+=data[cursor]+1
  cursor+=1
 return found
if case=='pixels':
 source=portal('palette');out=root/'exact';invoke('render','portal',source,out,['--strict-lossless'])
 manifest=json.loads((out/'manifest.json').read_text());assert manifest['fidelity']['gif_rgb']=='EXACT'
 assert manifest['fidelity']['master_rgb']=='EXACT'
 assert manifest['fidelity']['master_decoded_timing'][1]['pts']==123456
 assert manifest['fidelity']['master_decoded_timing'][1]['timebase']=='1/1000000'
 assert manifest['fidelity']['gif_decoded_timing'][1]['pts']==12
 assert manifest['fidelity']['gif_decoded_timing'][1]['timebase']=='1/100'
 assert (source/'observation.json').read_bytes()==(out/'source/observation.json').read_bytes()
 invoke('check','portal',out,root/'verified',['--strict-lossless'])
 wrong=root/'wrong-geometry';shutil.copytree(out,wrong)
 linear=b''.join(bytes(v for y in range(16) for x in range(32) for v in
  ((255,0,0) if (x+y+frame)%2 else (0,0,255))) for frame in range(2));write(root/'wrong.rgb',linear)
 subprocess.run([str(ffmpeg),'-nostdin','-v','error','-y','-f','rawvideo','-pixel_format','rgb24',
  '-video_size','16x32','-framerate','1000000/123456','-i',str(root/'wrong.rgb'),'-c:v','ffv1','-pix_fmt','bgr0',
  '-enc_time_base','1:1000000','-f','nut',str(wrong/'master.nut')],check=True,timeout=10)
 changed=json.loads((wrong/'manifest.json').read_text());changed['files']['master.nut']={'sha256':sha(wrong/'master.nut'),'bytes':(wrong/'master.nut').stat().st_size}
 observed=subprocess.run([str(ffmpeg),'-nostdin','-v','error','-copyts','-i',str(wrong/'master.nut'),
  '-fps_mode','passthrough','-c:v','rawvideo','-pix_fmt','rgb24','-enc_time_base','-1','-f','framehash','-hash','sha256','-'],
  check=True,capture_output=True,text=True,timeout=10).stdout.splitlines()
 assert '#dimensions 0: 16x32' in observed
 timebase=next(line.removeprefix('#tb 0: ') for line in observed if line.startswith('#tb 0: '))
 frames=[[field.strip() for field in line.split(',')] for line in observed if not line.startswith('#')]
 assert [row[5] for row in frames]==changed['fidelity']['master_rgb_sha256']==changed['fidelity']['source_rgb_sha256']
 measured=[{'pts':int(row[2]),'timebase':timebase} for row in frames]
 assert measured==changed['fidelity']['master_decoded_timing']
 changed['fidelity']['master_decoded_timing']=measured
 (wrong/'manifest.json').write_text(json.dumps(changed)+'\n')
 invoke('check','portal',wrong,root/'wrong-geometry-check',expect=1,reason='DECODED_GEOMETRY_MISMATCH')
 negative=portal('negative-time');value=json.loads((negative/'observation.json').read_text());value['frames'][0]['elapsed_ms']=-.5
 (negative/'observation.json').write_text(json.dumps(value)+'\n')
 invoke('render','portal',negative,root/'negative-time-refused',expect=1,reason='SCREENSHOT_TIMING_OR_PATH')
 colorful=portal('colorful',True);quantized=root/'quantized';invoke('render','portal',colorful,quantized)
 assert json.loads((quantized/'manifest.json').read_text())['fidelity']['gif_rgb']=='QUANTIZED'
 invoke('render','portal',colorful,root/'strict-refused',['--strict-lossless'],1,'GIF_RGB_QUANTIZED')
 invoke('check','portal',quantized,root/'quantized-check',['--strict-lossless'],1,'GIF_RGB_QUANTIZED')
 # Irregular schedule, including a25ms half-centisecond tie, must preserve native cadence.
 irregular=portal('irregular',elapsed=[100.123,125.123,175.623,368.956,1048.956]);timed=root/'timed'
 invoke('render','portal',irregular,timed,['--strict-lossless'])
 current=json.loads((timed/'manifest.json').read_text());fidelity=current['fidelity']
 assert [row['pts'] for row in fidelity['master_decoded_timing']]==[0,25000,75500,268833,948833]
 assert [row['pts'] for row in fidelity['gif_decoded_timing']]==[0,3,8,27,95]
 assert fidelity['gif_native_delays_cs']==[3,5,19,68,68]
 assert fidelity['final_frame_hold']=={'policy':'REPEAT_LAST_GAP_OR_SINGLE_100MS','delay_cs':68,'source_observed':False}
 invoke('check','portal',timed,root/'timed-check',['--strict-lossless'])
 for kind,reason in [('origin','MASTER_TIMING_ORIGIN'),('master','MASTER_SOURCE_TIMING_DRIFT'),('gif','GIF_SOURCE_TIMING_DRIFT'),('tail','GIF_NATIVE_DELAY_DRIFT')]:
  wrong=root/('retimed-'+kind);shutil.copytree(timed,wrong)
  item='master.nut' if kind in ['origin','master'] else 'derivative.gif';target=wrong/item
  if kind=='origin':
   args=['-c:v','copy','-output_ts_offset','2','-f','nut']
  elif kind=='master':
   args=['-vf','setpts=N*0.5/TB','-c:v','ffv1','-pix_fmt','bgr0','-enc_time_base','1:1000000','-f','nut']
  elif kind=='gif':
   args=['-filter_complex','setpts=N*0.5/TB,split[a][b];[a]palettegen=reserve_transparent=0[p];[b][p]paletteuse=dither=none',
    '-enc_time_base','1:100','-final_delay','50','-loop','0']
  else:
   data=bytearray(target.read_bytes());offset=gce_offsets(data)[-1];data[offset+4:offset+6]=struct.pack('<H',67);target.write_bytes(data)
  if kind!='tail':
   subprocess.run([str(ffmpeg),'-nostdin','-v','error','-y','-threads','1','-filter_threads','1','-filter_complex_threads','1',
    '-i',str(timed/'master.nut'),'-fps_mode','passthrough',*args,'-threads','1',str(target)],check=True,timeout=10)
  claimed=json.loads((wrong/'manifest.json').read_text())
  measured,rgb=observation(target);label='master' if kind in ['origin','master'] else 'gif'
  if kind=='origin':
   assert [row['pts'] for row in measured]==[row['pts']+2000000 for row in fidelity['master_decoded_timing']]
  assert rgb==claimed['fidelity'][label+'_rgb_sha256']==claimed['fidelity']['source_rgb_sha256']
  claimed['fidelity'][label+'_decoded_timing']=measured
  claimed['files'][item]={'sha256':sha(target),'bytes':target.stat().st_size}
  if kind not in ['origin','master']:
   raw=target.read_bytes();delays=[struct.unpack('<H',raw[x+4:x+6])[0] for x in gce_offsets(raw)]
   claimed['fidelity']['gif_native_delays_cs']=delays;claimed['fidelity']['final_frame_hold']['delay_cs']=delays[-1]
  (wrong/'manifest.json').write_text(json.dumps(claimed)+'\n')
  invoke('check','portal',wrong,root/('retimed-'+kind+'-check'),expect=1,reason=reason)
 for interval in [1,10]:
  source=portal('short-'+str(interval),elapsed=[0,interval])
  invoke('render','portal',source,root/('short-'+str(interval)+'-refused'),expect=1,reason='GIF_TIMING_UNREPRESENTABLE')
 single=portal('single',elapsed=[10]);single_out=root/'single-out';invoke('render','portal',single,single_out)
 one=json.loads((single_out/'manifest.json').read_text());assert one['fidelity']['gif_native_delays_cs']==[10]
 invoke('check','portal',single_out,root/'single-check')
 parse=runpy.run_path(str(implementation))['gif_delays'];raw=(timed/'derivative.gif').read_bytes();offset=gce_offsets(raw)[0]
 comment=b'\x21\xfe\x08'+b'\x21\xf9\x04\x00\xff\xff\x00\x00'+b'\x00'
 assert parse(raw[:-1]+comment+raw[-1:])==[3,5,19,68,68]
 for bad in [raw[:offset]+raw[offset+8:],raw[:offset]+raw[offset:offset+8]+raw[offset:],raw[:-1],
             raw[:-1]+b'\x21\x01\x00'+raw[-1:]]:
  try:parse(bad)
  except ValueError:pass
  else:raise AssertionError('ambiguous/truncated/missing/unsupported control accepted')
elif case=='custody':
 source=portal('source');out=root/'generation';invoke('render','portal',source,out)
 invoke('check','portal',out,root/'bad-manifest',expect=1,reason='MANIFEST_HASH_MISMATCH',digest='0'*64)
 invoke('render','portal',source,root/'bad-source',expect=2,reason='SOURCE_HASH_MISMATCH',impl_hash='0'*64)
 tool=root/'canary';write(tool,('#!'+str(python)+'\nfrom pathlib import Path\nPath('+repr(str(root/'executed'))+').touch()\n').encode());tool.chmod(0o700)
 invoke('render','portal',source,root/'bad-tool',expect=1,reason='TOOL_HASH_MISMATCH',executable=tool,tool_hash='0'*64)
 assert not (root/'executed').exists()
 tool2=root/'slow';write(tool2,('#!'+str(python)+'\nimport time\ntime.sleep(10)\n').encode());tool2.chmod(0o700)
 begin=time.monotonic();invoke('render','portal',source,root/'timed-out',['--timeout','0.1'],1,'TOOL_TIMEOUT',executable=tool2)
 assert time.monotonic()-begin<3
 interrupted=root/'interrupted';pidfile=root/'child.pid'
 tool3=root/'interruptible';write(tool3,('#!'+str(python)+'\nimport os,time\nfrom pathlib import Path\nPath('+repr(str(pidfile))+').write_text(str(os.getpid()))\ntime.sleep(20)\n').encode());tool3.chmod(0o700)
 command=['bash',str(hub/'scripts/demo-gif-render.sh'),str(python),sha(python),sha(implementation),
  '--kind','portal','--input',str(source),'--output',str(interrupted),'--input-sha256',sha(source/'observation.json'),
  '--ffmpeg',str(tool3),'--ffmpeg-sha256',sha(tool3)]
 child=subprocess.Popen(command,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
 deadline=time.monotonic()+5
 while not pidfile.exists() and time.monotonic()<deadline:time.sleep(.01)
 assert pidfile.exists();child.send_signal(signal.SIGTERM);stdout,stderr=child.communicate(timeout=3)
 assert child.returncode!=0 and b'RENDER_INTERRUPTED:15' in stderr
 assert (interrupted/'failure.json').exists() and not (interrupted/'manifest.json').exists()
 try:os.kill(int(pidfile.read_text()),0)
 except ProcessLookupError:pass
 else:raise AssertionError('owned tool survived interruption')
 try:os.killpg(int(pidfile.read_text()),0)
 except ProcessLookupError:pass
 else:raise AssertionError('owned tool group survived interruption')
 for interrupted_case in [False,True]:
  prefix='fork-interrupted' if interrupted_case else 'fork-exit';pid=root/(prefix+'.pid');sentinel=root/(prefix+'.survived');tool4=root/(prefix+'.tool')
  script='#!'+str(python)+'\nimport os,time\nfrom pathlib import Path\npid=os.fork()\nif pid==0:\n Path('+repr(str(pid))+').write_text(str(os.getpid()))\n time.sleep(3)\n Path('+repr(str(sentinel))+').touch()\n os._exit(0)\n'
  script+='while not Path('+repr(str(pid))+').exists(): time.sleep(.01)\n'
  if interrupted_case:script+='time.sleep(20)\n'
  write(tool4,script.encode());tool4.chmod(0o700);dest=root/prefix
  args=command.copy();args[args.index('--output')+1]=str(dest);args[args.index('--ffmpeg')+1]=str(tool4);args[args.index('--ffmpeg-sha256')+1]=sha(tool4)
  proc=subprocess.Popen(args,stdout=subprocess.PIPE,stderr=subprocess.PIPE);deadline=time.monotonic()+5
  while not pid.exists() and time.monotonic()<deadline:time.sleep(.01)
  assert pid.exists()
  if interrupted_case:proc.send_signal(signal.SIGTERM)
  stdout,stderr=proc.communicate(timeout=5);assert proc.returncode!=0 and not (dest/'manifest.json').exists()
  status=Path('/proc')/pid.read_text()/'stat'
  assert not status.exists() or status.read_text().split(') ',1)[1].split()[0]=='Z','same-group descendant still runs'
  assert not sentinel.exists()
 original=(out/'source/frames/0.png').read_bytes();(out/'source/frames/0.png').write_bytes(original+b'x')
 invoke('check','portal',out,root/'corrupt-output',expect=1,reason='GENERATION_ARTIFACT_DRIFT')
 before=(source/'observation.json').read_bytes();expected=sha(source/'observation.json')
 (source/'observation.json').write_bytes(before+b' ')
 invoke('render','portal',source,root/'corrupt-input',expect=1,reason='CAPTURE_HASH_MISMATCH',digest=expected)
 sentinel=root/'occupied';sentinel.mkdir(mode=0o700);write(sentinel/'keep',b'keep')
 invoke('render','portal',source,sentinel,expect=1)
 assert (sentinel/'keep').read_bytes()==b'keep'
elif case=='terminal':
 source=root/'terminal';source.mkdir(mode=0o700)
 cast=(json.dumps({'version':2,'width':40,'height':6})+'\n'+json.dumps([0.1,'o','Bullet Farm\r\n'])+'\n'+json.dumps([0.3,'o','Exact retained cast\r\n'])+'\n').encode()
 write(source/'session.cast',cast);write(source/'session.cast.raw',b'Bullet Farm\r\nExact retained cast\r\n');write(source/'transcript.txt',b'Bullet Farm\nExact retained cast\n')
 artifacts={key:{'sha256':sha(source/file),'bytes':(source/file).stat().st_size} for key,file in
  [('cast','session.cast'),('raw','session.cast.raw'),('transcript','transcript.txt')]}
 js(source/'session.cast.result.json',{'schema_version':'bullet.native-capture.v1','child_started':True,
  'owned_group_gone':True,'recorder_exit':0,'stop_reason':'CHILD_EXIT','completion':'UNVERIFIED',
  'bullet_live_admission':False,'artifacts':artifacts})
 out=root/'terminal-gif';invoke('render','terminal',source,out)
 result=json.loads((out/'manifest.json').read_text());assert result['closure_verified'] is False
 assert result['fidelity']['master_rgb']=='NOT_AVAILABLE'
 assert (out/'source/session.cast').read_bytes()==cast
 invoke('check','terminal',out,root/'terminal-check')
 invoke('render','terminal',source,root/'strict-terminal',['--strict-lossless'],1,'NO_ORIGINAL_TERMINAL_RGB_MASTER')
 invoke('render','terminal',source,root/'strict-font',['--strict-font-closure'],1,'FONT_CLOSURE_NOT_ADMITTED')
else:raise AssertionError(case)
"#;

fn fixture(case: &str) {
    let directory = tempfile::tempdir().expect("fixture directory");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o700))
            .expect("private fixture permissions");
    }
    let result = Command::new("python3")
        .args(["-I", "-c", FIXTURE, env!("CARGO_MANIFEST_DIR")])
        .arg(directory.path())
        .arg(case)
        .output()
        .expect("Python3 and supplied local FFmpeg/agg are required for this media fixture");
    assert!(
        result.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
}

#[test]
fn portal_rgb_master_and_palette_quantization_are_distinguished() {
    fixture("pixels");
}

#[test]
fn corrupt_subjects_tool_mismatch_timeout_and_output_collision_refuse() {
    fixture("custody");
}

#[test]
fn terminal_cast_is_preserved_without_inventing_original_rgb_or_font_closure() {
    fixture("terminal");
}
