import React, { useEffect, useState } from 'react'
import { invoke } from '@tauri-apps/api/tauri'

function App(){
  const [devices, setDevices] = useState([])
  const [selected, setSelected] = useState(null)
  const [deviceInfo, setDeviceInfo] = useState('')
  const [log, setLog] = useState('')
  const [backupFolder, setBackupFolder] = useState('~/RamdiskBackups')
  const [chip, setChip] = useState('A12')
  const [extraArgs, setExtraArgs] = useState('--enable-ssh')

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
    return ()=>clearInterval(interval)
  },[])

  async function showInfo(udid){
    try{
      const info = await invoke('get_device_info', { udid })
      setDeviceInfo(info)
    }catch(e){ setDeviceInfo('Failed: '+JSON.stringify(e)) }
  }

  async function doBoot(){
    if(!selected) return alert('Select device')
    setLog(l=>'Starting boot...\n'+l)
    const out = await invoke('boot_ramdisk', { udid: selected.udid, chip, extra_args: extraArgs })
    setLog(l=>out + '\n' + l)
  }

  async function doBackup(){
    if(!selected) return alert('Select device')
    const out = await invoke('backup_files', { udid: selected.udid, dest_folder: backupFolder, extra_args: '' })
    setLog(l=>out + '\n' + l)
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

            <div className="mt-4 flex gap-2">
              <button onClick={doBoot} className="px-4 py-2 bg-green-600 text-white rounded">Boot Ramdisk</button>
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
