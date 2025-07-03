import { ReactNode } from 'react'
import Sidebar from './Sidebar'
import Player from '@components/player/Player'

interface LayoutProps {
  children: ReactNode
}

export default function Layout({ children }: LayoutProps) {
  return (
    <div className="flex h-screen bg-gray-100 dark:bg-gray-900">
      {/* 侧边栏 */}
      <Sidebar />
      
      {/* 主内容区 */}
      <main className="flex-1 overflow-auto">
        <div className="container mx-auto px-4 py-8">
          {children}
        </div>
      </main>
      
      {/* 播放器 */}
      <div className="fixed bottom-0 left-0 right-0">
        <Player />
      </div>
    </div>
  )
}