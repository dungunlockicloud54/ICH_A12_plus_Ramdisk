import React, { useEffect, useState } from 'react'
import { invoke } from '@tauri-apps/api/tauri'
import { listen } from '@tauri-apps/api/event'

function App(){
  const [devices, setDevices] = useState([])
  const [selected, setSelected] = useState(null)
  const [deviceInfo, setDeviceInfo] = useState('')
  const [log, setLog] = useState('')
  const [backupFolder, setBackupFolder] = useState('~/RamdiskBackups')
  const [chip, setChip] = useState('A12')
  const [extraArgs, setExtraArgs] = useState('--enable-ssh')
  const [host, setHost] = useState('127.0.0.1')
  const [port, setPort] = useState(2222)
  const [user, setUser] = useState('root')
  const [password, setPassword] = useState('alpine')
  const [overwrite, setOverwrite] = useState(true)
  const [isBooting, setIsBooting] = useState(false)
  const [saveKeychain, setSaveKeychain] = useState(true)

  async function refreshDevices(){
    try{
      const res = await invoke('list_devices')
      setDevices(res)
    }catch(e){
      setLog(l => l + '\n' + JSON.stringify(e))
    }
  }

  useEffect(()=>{
    refreshDevices()
    const interval = setInterval(refreshDevices, 5000)

    // load persisted config
    invoke('load_config').then(cfg => {
      if(cfg) {
        try{
          const parsed = JSON.parse(cfg)
          if(parsed.backupFolder) setBackupFolder(parsed.backupFolder)
          if(parsed.host) setHost(parsed.host)
          if(parsed.port) setPort(parsed.port)
          if(parsed.user) setUser(parsed.user)
          if(parsed.overwrite !== undefined) setOverwrite(parsed.overwrite)
          if(parsed.chip) setChip(parsed.chip)
        }catch(e){ console.warn('failed to parse config', e) }
      }
    }).catch(e=>console.warn('load_config failed', e))

    // event listeners for boot logs
    let unlistenBootLog, unlistenBootFinished
    listen('boot-log', event => {
      const payload = event.payload;
      setLog(l => payload + '\n' + l)
    }).then(f => { unlistenBootLog = f })
    listen('boot-finished', event => {
      const payload = event.payload;
      setLog(l => ('BOOT FINISHED: ' + payload) + '\n' + l)
      setIsBooting(false)
    }).then(f => { unlistenBootFinished = f })

    return ()=>{ clearInterval(interval); if(unlistenBootLog) unlistenBootLog(); if(unlistenBootFinished) unlistenBootFinished(); }
  },[])

  async function showInfo(udid){
    try{
      const info = await invoke('get_device_info', { udid })
      setDeviceInfo(info)
    }catch(e){ setDeviceInfo('Failed: '+JSON.stringify(e)) }
  }

  async function saveSettings(){
    const cfg = { backupFolder, host, port, user, overwrite, chip }
    try{
      await invoke('save_config', { config_json: JSON.stringify(cfg) })
      setLog(l=> 'Saved config\n' + l)
    }catch(e){ setLog(l=>'Failed to save config: '+JSON.stringify(e)+'\n'+l) }
  }

  async function doBoot(){
    if(!selected) return alert('Select device')
    setLog(l=>'Starting boot...\n'+l)
    setIsBooting(true)
    try{
      await invoke('boot_ramdisk', { udid: selected.udid, chip, extra_args: extraArgs })
    }catch(e){ setLog(l=>'Failed to start boot: '+JSON.stringify(e)+'\n'+l); setIsBooting(false) }
  }

  async function doBackup(){
    if(!selected) return alert('Select device')
    setLog(l=>'Starting backup...\n'+l)
    try{
      if(saveKeychain){
        await invoke('save_ssh_password', { host, port, user, password })
      }
      const out = await invoke('backup_files', { udid: selected.udid, dest_folder: backupFolder, host, port, user, password: null, use_keychain: saveKeychain, overwrite })
      setLog(l=>out + '\n' + l)
    }catch(e){ setLog(l=>'Backup failed: '+JSON.stringify(e)+'\n'+l) }
  }

  return (
    <div className="min-h-screen bg-gray-100 dark:bg-gray-900 text-gray-900 dark:text-gray-100">
      <div className="max-w-5xl mx-auto p-6">
        <header className="flex items-center justify-between mb-6">
          <h1 className="text-2xl font-semibold">DienThoai88 Ramdisk Tool</h1>
          <div>
            <button onClick={refreshDevices} className="px-3 py-1 bg-blue-600 text-white rounded">Refresh</button>
          </div>
        </header>

        <section className="grid grid-cols-3 gap-4">
          <div className="col-span-1 bg-white dark:bg-gray-800 p-4 rounded shadow">
            <h2 className="font-medium">Devices</h2>
            <ul className="mt-2">
              {devices && devices.length>0 ? devices.map(d=> (
                <li key={d.udid} className={`p-2 border rounded mt-2 cursor-pointer ${selected && selected.udid===d.udid? 'bg-blue-50 dark:bg-blue-900':''}`} onClick={()=>{setSelected(d); showInfo(d.udid)}}>
                  <div className="font-mono text-sm">{d.udid}</div>
                </li>
              )) : <li className="text-sm text-gray-500">No devices</li>}
            </ul>
          </div>

          <div className="col-span-2 bg-white dark:bg-gray-800 p-4 rounded shadow">
            <h2 className="font-medium">Device Info</h2>
            <pre className="mt-2 text-xs max-h-48 overflow-auto bg-gray-50 dark:bg-gray-900 p-2 rounded">{deviceInfo || 'Select a device to show info'}</pre>

            <div className="mt-4 grid grid-cols-3 gap-2">
              <select value={chip} onChange={e=>setChip(e.target.value)} className="p-2 border rounded col-span-1">
                <option>A12</option>
                <option>A13</option>
                <option>A14</option>
                <option>A15</option>
                <option>A16</option>
              </select>
              <input value={extraArgs} onChange={e=>setExtraArgs(e.target.value)} className="p-2 border rounded col-span-1" placeholder="Extra args for ramdisk tool" />
              <input value={backupFolder} onChange={e=>setBackupFolder(e.target.value)} className="p-2 border rounded col-span-1" />
            </div>

            <div className="mt-4 grid grid-cols-6 gap-2">
              <input value={host} onChange={e=>setHost(e.target.value)} className="p-2 border rounded col-span-1" placeholder="host" />
              <input type="number" value={port} onChange={e=>setPort(Number(e.target.value))} className="p-2 border rounded col-span-1" placeholder="port" />
              <input value={user} onChange={e=>setUser(e.target.value)} className="p-2 border rounded col-span-1" placeholder="user" />
              <input value={password} onChange={e=>setPassword(e.target.value)} className="p-2 border rounded col-span-1" placeholder="password" />
              <label className="flex items-center col-span-2"><input type="checkbox" checked={overwrite} onChange={e=>setOverwrite(e.target.checked)} className="mr-2"/>Overwrite existing</label>
            </div>

            <div className="mt-2 flex items-center gap-2">
              <label className="flex items-center"><input type="checkbox" checked={saveKeychain} onChange={e=>setSaveKeychain(e.target.checked)} className="mr-2"/>Save password to Keychain</label>
              <button onClick={saveSettings} className="px-3 py-1 bg-gray-600 text-white rounded">Save Settings</button>
            </div>

            <div className="mt-4 flex gap-2">
              <button disabled={isBooting} onClick={doBoot} className="px-4 py-2 bg-green-600 text-white rounded">{isBooting? 'Booting...':'Boot Ramdisk'}</button>
              <button onClick={doBackup} className="px-4 py-2 bg-yellow-600 text-white rounded">Backup Active Files</button>
              <button onClick={()=>navigator.clipboard.writeText(deviceInfo)} className="px-4 py-2 bg-gray-600 text-white rounded">Copy Info</button>
            </div>

            <h3 className="mt-6">Logs</h3>
            <textarea readOnly value={log} className="w-full h-48 mt-2 p-2 bg-black text-white text-xs rounded"></textarea>
          </div>
        </section>
      </div>
    </div>
  )
}

export default App
