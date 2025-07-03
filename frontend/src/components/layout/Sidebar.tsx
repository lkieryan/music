import { NavLink } from 'react-router-dom'
import { Home, Library, Search, Settings } from 'lucide-react'

const navigation = [
  { name: '首页', icon: Home, href: '/' },
  { name: '音乐库', icon: Library, href: '/library' },
  { name: '搜索', icon: Search, href: '/search' },
  { name: '设置', icon: Settings, href: '/settings' },
]

export default function Sidebar() {
  return (
    <div className="w-64 bg-white dark:bg-gray-800 border-r border-gray-200 dark:border-gray-700">
      {/* Logo */}
      <div className="h-16 flex items-center px-4">
        <h1 className="text-xl font-bold text-gray-800 dark:text-white">
          MoeKoe Music
        </h1>
      </div>
      
      {/* 导航菜单 */}
      <nav className="px-2 py-4">
        {navigation.map((item) => (
          <NavLink
            key={item.name}
            to={item.href}
            className={({ isActive }) =>
              `flex items-center px-4 py-2 mt-2 text-gray-600 dark:text-gray-300 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 ${
                isActive ? 'bg-gray-100 dark:bg-gray-700' : ''
              }`
            }
          >
            <item.icon className="w-5 h-5 mr-3" />
            <span>{item.name}</span>
          </NavLink>
        ))}
      </nav>
      
      {/* 播放列表 */}
      <div className="px-4 py-4 border-t border-gray-200 dark:border-gray-700">
        <h2 className="text-sm font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-wider">
          播放列表
        </h2>
        {/* TODO: 添加播放列表 */}
      </div>
    </div>
  )
}