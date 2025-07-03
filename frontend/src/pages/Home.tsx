import { useEffect, useState } from 'react'
import { invoke } from '@tauri-apps/api/tauri'

interface Stats {
  totalSongs: number
  totalArtists: number
  totalAlbums: number
  totalDuration: number
  totalSize: number
}

export default function Home() {
  const [stats, setStats] = useState<Stats | null>(null)
  
  useEffect(() => {
    // TODO: 从后端获取音乐库统计信息
  }, [])
  
  return (
    <div>
      <h1 className="text-2xl font-bold text-gray-900 dark:text-white mb-8">
        欢迎使用 MoeKoe Music
      </h1>
      
      {/* 统计信息 */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
        <div className="bg-white dark:bg-gray-800 p-6 rounded-lg shadow-sm">
          <h3 className="text-lg font-medium text-gray-900 dark:text-white">
            音乐总数
          </h3>
          <p className="text-3xl font-bold text-primary mt-2">
            {stats?.totalSongs ?? 0}
          </p>
        </div>
        
        <div className="bg-white dark:bg-gray-800 p-6 rounded-lg shadow-sm">
          <h3 className="text-lg font-medium text-gray-900 dark:text-white">
            艺术家
          </h3>
          <p className="text-3xl font-bold text-primary mt-2">
            {stats?.totalArtists ?? 0}
          </p>
        </div>
        
        <div className="bg-white dark:bg-gray-800 p-6 rounded-lg shadow-sm">
          <h3 className="text-lg font-medium text-gray-900 dark:text-white">
            专辑
          </h3>
          <p className="text-3xl font-bold text-primary mt-2">
            {stats?.totalAlbums ?? 0}
          </p>
        </div>
        
        <div className="bg-white dark:bg-gray-800 p-6 rounded-lg shadow-sm">
          <h3 className="text-lg font-medium text-gray-900 dark:text-white">
            总时长
          </h3>
          <p className="text-3xl font-bold text-primary mt-2">
            {Math.floor((stats?.totalDuration ?? 0) / 3600)}小时
          </p>
        </div>
      </div>
      
      {/* 最近播放 */}
      <section className="mb-8">
        <h2 className="text-xl font-bold text-gray-900 dark:text-white mb-4">
          最近播放
        </h2>
        {/* TODO: 添加最近播放列表 */}
      </section>
      
      {/* 收藏歌单 */}
      <section className="mb-8">
        <h2 className="text-xl font-bold text-gray-900 dark:text-white mb-4">
          收藏歌单
        </h2>
        {/* TODO: 添加收藏歌单列表 */}
      </section>
    </div>
  )
}