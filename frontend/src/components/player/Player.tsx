import { useState } from 'react'
import { Play, Pause, SkipBack, SkipForward, Volume2, Heart } from 'lucide-react'
import usePlayer from '@hooks/usePlayer'

export default function Player() {
  const [volume, setVolume] = useState(1)
  const [isFavorite, setIsFavorite] = useState(false)
  
  // TODO: 使用 usePlayer hook 获取播放状态和控制方法
  
  return (
    <div className="h-24 bg-white dark:bg-gray-800 border-t border-gray-200 dark:border-gray-700">
      <div className="container mx-auto h-full flex items-center px-4">
        {/* 当前歌曲信息 */}
        <div className="flex-shrink-0 w-1/4">
          <div className="flex items-center">
            <img
              src="placeholder.jpg"
              alt="Album Cover"
              className="w-16 h-16 rounded-lg shadow-lg"
            />
            <div className="ml-4">
              <h3 className="text-gray-800 dark:text-white font-medium">
                歌曲名称
              </h3>
              <p className="text-gray-500 dark:text-gray-400 text-sm">
                艺术家
              </p>
            </div>
          </div>
        </div>
        
        {/* 播放控制 */}
        <div className="flex-grow flex flex-col items-center">
          <div className="flex items-center space-x-6">
            <button className="text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-white">
              <SkipBack className="w-6 h-6" />
            </button>
            <button className="w-10 h-10 rounded-full bg-primary flex items-center justify-center text-white">
              <Play className="w-6 h-6" />
            </button>
            <button className="text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-white">
              <SkipForward className="w-6 h-6" />
            </button>
          </div>
          
          {/* 进度条 */}
          <div className="w-full mt-4 flex items-center space-x-4 text-sm">
            <span className="text-gray-500">0:00</span>
            <div className="flex-grow">
              <div className="h-1 bg-gray-200 dark:bg-gray-700 rounded-full">
                <div
                  className="h-1 bg-primary rounded-full"
                  style={{ width: '0%' }}
                />
              </div>
            </div>
            <span className="text-gray-500">0:00</span>
          </div>
        </div>
        
        {/* 音量和收藏 */}
        <div className="flex-shrink-0 w-1/4 flex items-center justify-end space-x-4">
          <button
            className={`text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-white ${
              isFavorite ? 'text-red-500 dark:text-red-400' : ''
            }`}
            onClick={() => setIsFavorite(!isFavorite)}
          >
            <Heart className="w-5 h-5" fill={isFavorite ? 'currentColor' : 'none'} />
          </button>
          
          <div className="flex items-center space-x-2">
            <Volume2 className="w-5 h-5 text-gray-500" />
            <input
              type="range"
              min="0"
              max="1"
              step="0.01"
              value={volume}
              onChange={(e) => setVolume(parseFloat(e.target.value))}
              className="w-24"
            />
          </div>
        </div>
      </div>
    </div>
  )
}