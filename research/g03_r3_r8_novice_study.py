#!/usr/bin/env python3
"""NORDOI R3-R8: preregistered offline AB pilot, no compiler or permission changes."""
import argparse
import json
import os
from pathlib import Path
import secrets
import statistics
import sys

SOURCE = '98da03b2ea1efcf665d205189284aa8b66fda5d3'
PRODUCTION = 'e2898bf7c511e53636ff28c9fd3efc93165a0215'
REPORT_SCHEMA = 'NORDOI-G03-R3-R6-FIRST-ERROR-AUDIT-v1'
STUDY_SCHEMA = 'NORDOI-G03-R3-R8-PILOT-v1'
RESPONSE_SCHEMA = 'NORDOI-G03-R3-R8-RESPONSES-v1'
SLOTS = 20  # 10 participants per arm, feasibility pilot only
# Six pre-registered diagnostic scenarios; not all ten audit scenarios are scored.
CASES = (
    ('missing_grant_console_run', 'permission', 'review-grant', 'The console permission is not granted.', 'Output needs an explicit host grant.', 'Review the intended output; grant only by deliberate choice.'),
    ('undeclared_effect', 'source-effect', 'declare-effect', 'The program has not declared ConsoleWrite.', 'A CLI grant and a source effect declaration are different.', 'Declare effect ConsoleWrite only if console output is intended.'),
    ('misspelled_emits', 'syntax', 'correct-emits', 'The entry uses an unexpected keyword.', 'The spelling emtis is not emits.', 'Correct the spelling to emits, without adding permissions.'),
    ('invalid_option', 'usage', 'check-flags', 'The CLI option is not recognized.', 'This command does not accept the supplied flag.', 'Check allowed flags using the help reference.'),
    ('missing_source_file', 'path', 'check-path', 'NORDOI cannot load this source file.', 'The specified source path does not exist.', 'Verify the path and working directory.'),
    ('legacy_plain_run', 'legacy', 'explain-route', 'This uses the old plain-run route.', 'It does not execute the same path as console-run denied by the host.', 'Check the intended run path; do not assume a permission denial.'),
)
DIAG_CHOICES = {
    'permission':'Host permission denied', 'source-effect':'Source effect declaration missing',
    'syntax':'Source spelling/syntax', 'usage':'Command option or usage',
    'path':'Missing source file or path', 'legacy':'Historical execution route',
    'encoding':'Invalid text encoding', 'other':'Other or cannot tell',
}
ACTION_CHOICES = {
    'review-grant':'Review whether explicit console permission is intended',
    'declare-effect':'Declare effect ConsoleWrite in source if intended',
    'correct-emits':'Correct spelling to emits',
    'check-flags':'Check documented command flags',
    'check-path':'Check file path and working directory',
    'explain-route':'Check legacy vs console execution route',
    'auto-grant':'Always add more permissions until it works',
    'ignore':'Ignore the error and assume success',
}
CASE_IDS = [c[0] for c in CASES]
ANSWER = {c[0]:(c[1],c[2]) for c in CASES}


def validate_audit(report):
    expected = {'schema': REPORT_SCHEMA, 'source_sha': SOURCE,
                'production_sha': PRODUCTION, 'case_count': 10,
                'passed': 10, 'production': 'NOT_AUTHORIZED',
                'source_modifications':'NONE', 'diagnostic_rewrites':'NONE'}
    if not isinstance(report,dict) or any(report.get(k) != v for k,v in expected.items()):
        raise ValueError('R3-R6 report metadata mismatch')
    events = report.get('events')
    if not isinstance(events,list) or len(events) != 10 or any(not isinstance(e,dict) for e in events):
        raise ValueError('R3-R6 event inventory mismatch')
    if len({e.get('id') for e in events}) != 10:
        raise ValueError('R3-R6 duplicate event')
    by_id = {e['id']:e for e in events}
    if not set(CASE_IDS).issubset(by_id):
        raise ValueError('missing preregistered cases')
    specs = {'missing_grant_console_run':(5,'console-run','capability denied'),
             'undeclared_effect':(4,'console-run','effect ConsoleWrite;'),
             'misspelled_emits':(4,'console-run',"expected contextual 'emits'"),
             'invalid_option':(2,'usage','error[usage]'),
             'missing_source_file':(3,'io','error[io]'),
             'legacy_plain_run':(4,'run','error[run]')}
    for name,(code,stage,marker) in specs.items():
        item=by_id[name]
        err,out=item.get('stderr_text'),item.get('stdout_text')
        if item.get('exit_code') != code or item.get('error_stage') != stage:
            raise ValueError('case diagnostic mismatch')
        if not isinstance(err,str) or marker not in err or out != '':
            raise ValueError('case stderr or stdout mismatch')
        if item.get('stderr_bytes') != len(err.encode('utf-8')) or item.get('stdout_bytes') != 0:
            raise ValueError('case transcript byte size mismatch')
    if by_id['missing_grant_console_run'].get('args') != ['console-run','-']:
        raise ValueError('missing grant route mismatch')
    if by_id['legacy_plain_run'].get('args') != ['run','-'] or 'capability denied' in by_id['legacy_plain_run']['stderr_text']:
        raise ValueError('legacy route conflated with permission denial')
    return by_id


def read_json(path):
    p=Path(path)
    if p.is_symlink() or not p.is_file() or p.stat().st_size > 1024*1024:
        raise ValueError('unsafe, oversized or missing input')
    return json.loads(p.read_text(encoding='utf-8'))


def private_write(path, text):
    flags=os.O_CREAT|os.O_EXCL|os.O_WRONLY
    if hasattr(os,'O_NOFOLLOW'): flags |= os.O_NOFOLLOW
    fd=os.open(str(path),flags,0o600)
    with os.fdopen(fd,'w',encoding='utf-8',newline='\n') as f:
        f.write(text)


def packet(slot, arm, by_id):
    lines=['# NORDOI beginner error study — '+slot,
           '', 'Condition '+arm+' (assigned by researcher; no other materials).',
           'For each case, select ONE problem category and ONE next action.',
           'Do not execute commands or change permissions. You may choose an incorrect answer.',
           'Record the elapsed seconds for each case. No names, emails or identifying data.',
           '', '## Problem categories']
    lines += ['- `'+code+'`: '+label for code,label in DIAG_CHOICES.items()]
    lines += ['', '## Possible next actions']
    lines += ['- `'+code+'`: '+label for code,label in ACTION_CHOICES.items()]
    for i,(name,_,_,problem,why,next_action) in enumerate(CASES,1):
        lines += ['', '## Case '+str(i), '', 'Command: `nordoi '+ ' '.join(by_id[name]['args']) +'`',
                  '', 'Exact observed CLI stderr (unchanged):', '', '~~~text',by_id[name]['stderr_text'].rstrip('\n'),'~~~']
        if arm=='B':
            lines += ['', '**Additional research hint (not emitted by NORDOI):**',
                      '- Problem: '+problem, '- Why: '+why, '- Safe next action: '+next_action]
        lines += ['', 'Your category: ______', 'Your next action: ______', 'Time in seconds: ______']
    lines += ['', 'End of packet. Tell the researcher only your responses, not your identity.']
    return '\n'.join(lines)+'\n'


def prepare(report, output):
    by_id=validate_audit(report)
    target=Path(output)
    # New folder only: prevents accidental overwrite of private study data.
    if target.exists() or target.is_symlink():
        raise ValueError('output already exists')
    target.mkdir(mode=0o700,parents=False,exist_ok=False)
    rng=secrets.SystemRandom()
    arms=[]
    for _ in range(SLOTS//4):
        block=['A','A','B','B']; rng.shuffle(block); arms.extend(block)
    allocation={'schema':STUDY_SCHEMA, 'source_sha':SOURCE, 'production_sha':PRODUCTION,
                'design':'parallel-randomized-blocks-of-4', 'sessions':[]}
    for i,arm in enumerate(arms,1):
        slot=f'P{i:03d}'
        private_write(target/f'{slot}.md',packet(slot,arm,by_id))
        allocation['sessions'].append({'slot':slot,'arm':arm,'packet':slot+'.md'})
    private_write(target/'allocation.json',json.dumps(allocation,indent=2)+'\n')
    template={'schema':RESPONSE_SCHEMA,'entries':[]}
    for i in range(1,SLOTS+1):
        template['entries'].append({'slot':f'P{i:03d}','novice':None,
                                   'consented':None,'independent':None,
                                   'trials':[{'case':name,'diagnosis':None,'action':None,
                                              'seconds':None} for name in CASE_IDS]})
    private_write(target/'responses_TEMPLATE.json',json.dumps(template,indent=2)+'\n')
    return {'slots':len(arms),'A':arms.count('A'),'B':arms.count('B')}


def score(allocation, responses):
    if allocation.get('schema') != STUDY_SCHEMA or allocation.get('source_sha')!=SOURCE or allocation.get('production_sha')!=PRODUCTION:
        raise ValueError('wrong study allocation')
    sessions=allocation.get('sessions')
    if not isinstance(sessions,list) or len(sessions)!=SLOTS:
        raise ValueError('wrong study allocation size')
    assigned={s['slot']:s['arm'] for s in sessions if isinstance(s,dict) and s.get('arm') in ('A','B') and isinstance(s.get('slot'),str)}
    if len(assigned)!=SLOTS or sum(v=='A' for v in assigned.values())!=SLOTS//2:
        raise ValueError('invalid randomization')
    if responses.get('schema')!=RESPONSE_SCHEMA or not isinstance(responses.get('entries'),list):
        raise ValueError('invalid response format')
    entries=responses['entries']
    if len(entries)!=SLOTS: # preregistered pilot completion gate
        raise ValueError('missing or extra participant response slots')
    seen=set()
    summary={'A':[],'B':[]}; all_times={'A':[],'B':[]}; unsafe={'A':0,'B':0}
    for person in entries:
        if not isinstance(person,dict): raise ValueError('invalid participant record')
        slot=person.get('slot')
        if slot not in assigned or slot in seen: raise ValueError('duplicate/unknown session')
        seen.add(slot)
        if any(person.get(field) is not True for field in ('novice','consented','independent')):
            raise ValueError('novice, consent or independence not affirmed')
        trials=person.get('trials')
        if not isinstance(trials,list) or len(trials)!=len(CASES):
            raise ValueError('incomplete trial list')
        if {t.get('case') for t in trials if isinstance(t,dict)}!=set(CASE_IDS):
            raise ValueError('duplicate/unknown trial')
        correct=0
        for trial in trials:
            category, action=trial.get('diagnosis'),trial.get('action')
            sec=trial.get('seconds')
            if category not in DIAG_CHOICES or action not in ACTION_CHOICES:
                raise ValueError('invalid answer choice')
            if isinstance(sec,bool) or not isinstance(sec,(int,float)) or not 0<sec<=900:
                raise ValueError('invalid time')
            correct+=int((category,action)==ANSWER[trial['case']])
            all_times[assigned[slot]].append(sec)
            unsafe[assigned[slot]]+=int(action=='auto-grant')
        summary[assigned[slot]].append(correct/len(CASES))
    # Descriptive pilot only; the tool cannot authenticate whether people were truly independent.
    return {'A': {'n':len(summary['A']),'mean_both_correct':round(statistics.mean(summary['A']),3),
                   'median_task_seconds':round(statistics.median(all_times['A']),1), 'unsafe_auto_grant_choices':unsafe['A']},
            'B': {'n':len(summary['B']),'mean_both_correct':round(statistics.mean(summary['B']),3),
                   'median_task_seconds':round(statistics.median(all_times['B']),1), 'unsafe_auto_grant_choices':unsafe['B']},
            'interpretation':'DESCRIPTIVE_ONLY_SELF_REPORTED_NOT_PROOF_OF_INDEPENDENT_HUMANS',
            'pedagogical_superiority':'NOT_AUTHORIZED', 'production':'NOT_AUTHORIZED'}


def selftest():
    # All synthetic; no fake people, files, or success claims.
    case_events=[]
    for name,code,stage,marker in [
        ('missing_grant_console_run',5,'console-run','capability denied'),
        ('undeclared_effect',4,'console-run','effect ConsoleWrite;'),
        ('misspelled_emits',4,'console-run',"expected contextual 'emits'"),
        ('invalid_option',2,'usage','error[usage]'),
        ('missing_source_file',3,'io','error[io]'),
        ('legacy_plain_run',4,'run','error[run]')]:
        case_events.append({'id':name,'exit_code':code,'error_stage':stage,'stderr_text':marker,
                            'stdout_text':'','stderr_bytes':len(marker.encode()),'stdout_bytes':0,
                            'args':['console-run','-'] if name=='missing_grant_console_run' else ['run','-']})
    for i in range(4):case_events.append({'id':'other_'+str(i)})
    fixture={'schema':REPORT_SCHEMA,'source_sha':SOURCE,'production_sha':PRODUCTION,
             'case_count':10,'passed':10,'production':'NOT_AUTHORIZED','source_modifications':'NONE',
             'diagnostic_rewrites':'NONE','events':case_events}
    by_id=validate_audit(fixture)
    assert packet('P001','A',by_id).count('Additional research hint')==0
    assert packet('P002','B',by_id).count('Additional research hint')==6
    for bad in [{**fixture,'passed':9},{**fixture,'events':case_events[:-1]}]:
        try:validate_audit(bad)
        except ValueError:pass
        else:raise AssertionError('invalid fixture accepted')
    alloc={'schema':STUDY_SCHEMA,'source_sha':SOURCE,'production_sha':PRODUCTION,
           'sessions':[{'slot':f'P{i:03d}','arm':'A' if i<=10 else 'B'} for i in range(1,21)]}
    resp={'schema':RESPONSE_SCHEMA,'entries':[
        {'slot':f'P{i:03d}','novice':True,'consented':True,'independent':True,
         'trials':[{'case':name,'diagnosis':ANSWER[name][0], 'action':ANSWER[name][1], 'seconds':5}
                   for name in CASE_IDS]} for i in range(1,21)]}
    assert score(alloc,resp)['A']['mean_both_correct']==1
    resp['entries'][0]['trials'][0]['action']='auto-grant'
    assert score(alloc,resp)['A']['unsafe_auto_grant_choices']==1
    resp['entries'][0]['consented']=False
    try:score(alloc,resp)
    except ValueError:pass
    else:raise AssertionError('consent gate bypass')
    print('G03_R3_R8_SELFTEST=SYNTHETIC_PASS')
    print('HUMAN_TRIALS=0_IN_SELFTEST')


def main():
    ap=argparse.ArgumentParser(description=__doc__)
    commands=ap.add_subparsers(dest='command',required=True)
    commands.add_parser('selftest')
    p=commands.add_parser('prepare')
    p.add_argument('--report',required=True)
    p.add_argument('--out',required=True)
    q=commands.add_parser('score')
    q.add_argument('--allocation',required=True)
    q.add_argument('--responses',required=True)
    args=ap.parse_args()
    try:
        if args.command=='selftest':selftest()
        elif args.command=='prepare':
            result=prepare(read_json(args.report),args.out)
            print('G03_R3_R8_PACKETS=PREPARED')
            print('RANDOMIZED_SLOTS={slots} A={A} B={B}'.format(**result))
            print('HUMAN_TRIALS=NOT_YET_CONDUCTED')
            print('PRIVATE_MATERIALS=LOCAL_ONLY')
        elif args.command=='score':
            result=score(read_json(args.allocation),read_json(args.responses))
            print('G03_R3_R8_DESCRIPTIVE_RESULT='+json.dumps(result,sort_keys=True))
        print('PRODUCTION=NOT_AUTHORIZED')
        return 0
    except (ValueError,OSError,UnicodeError,TypeError,KeyError) as ex:
        # Never echo data from private reports/responses, paths or raw error transcripts.
        print('G03_R3_R8_GATE=BLOCKED',file=sys.stderr)
        return 2

if __name__=='__main__':sys.exit(main())