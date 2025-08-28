import type { FC } from 'react'
import { useState, useEffect, useCallback } from 'react'
import { useTranslation } from 'react-i18next'
import { useNavigate, useLocation, useSearchParams } from 'react-router'
import { useAtom } from 'jotai'
import SearchIcon from '~/assets/icons/search-glass.svg?react'
import { useDesktopLayout } from '~/providers/layout-provider'
import { cn } from '~/lib/helper'
import { PRESET_COLORS } from '~/constants/gradient'
import { musicSearch } from '~/services/music-api'
import { 
  searchTermAtom, 
  searchLoadingAtom, 
  searchResultAtom, 
  searchErrorAtom, 
  lastSearchTermAtom 
} from '~/atoms/search'

export const Urlbar: FC = () => {
  const { t } = useTranslation('common')
  const { singleToolbar } = useDesktopLayout()
  const navigate = useNavigate()
  const location = useLocation()
  const [searchParams] = useSearchParams()
  
  // Manage state using atoms
  const [searchTerm, setSearchTerm] = useAtom(searchTermAtom)
  const [loadingState, setLoadingState] = useAtom(searchLoadingAtom)
  const [, setSearchResult] = useAtom(searchResultAtom)
  const [, setSearchError] = useAtom(searchErrorAtom)
  const [, setLastSearchTerm] = useAtom(lastSearchTermAtom)
  
  // Local UI state
  const [isFocused, setIsFocused] = useState(false)
  const [isExtended, setIsExtended] = useState(false)

  // Sync search term from URL to search bar (only when URL changes)
  useEffect(() => {
    if (location.pathname === '/search') {
      const queryParam = searchParams.get('q')
      if (queryParam) {
        setSearchTerm(queryParam)
      }
    } else {
      // Clear search term when leaving search page
      setSearchTerm('')
    }
  }, [location.pathname, searchParams, setSearchTerm])

  // Function to perform search
  const performSearch = useCallback(async (term: string) => {
    if (!term.trim()) return
    
    // Navigate to search page immediately
    navigate(`/search?q=${encodeURIComponent(term.trim())}`)
    
    try {
      setLoadingState('loading')
      setIsExtended(true)
      setSearchError(null)
      setLastSearchTerm(term.trim())
      
      // Call search API
      const result = await musicSearch(term.trim(), {
        types: ["Track"],
        page: { limit: 50, offset: 0, cursor: null }
      })
      
      // Save results to atoms
      setSearchResult(result)
      setLoadingState('success')
      
    } catch (error) {
      console.error('Search failed:', error)
      setSearchError(error instanceof Error ? error.message : 'Search failed')
      setLoadingState('error')
    } finally {
      setIsExtended(false)
      
      // Reset loading state
      setTimeout(() => {
        setLoadingState('idle')
      }, 1000)
    }
  }, [setLoadingState, setSearchResult, setSearchError, setLastSearchTerm, navigate])

  // Handle Enter key
  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter' && searchTerm.trim()) {
      e.preventDefault()
      performSearch(searchTerm)
    }
  }

  // Get gradient background style from preset colors
  const getBackgroundStyle = () => {
    if (isExtended) {
      const analogousPresets = PRESET_COLORS[1]
      const selectedPreset = analogousPresets[Math.floor(Math.random() * analogousPresets.length)]
      
      if ('colors' in selectedPreset && selectedPreset.colors) {
        const [color1, color2, color3] = selectedPreset.colors
        
        const rgba1 = color1.replace('rgb', 'rgba').replace(')', ', 0.25)')
        const rgba2 = color2.replace('rgb', 'rgba').replace(')', ', 0.35)')
        const rgba3 = color3.replace('rgb', 'rgba').replace(')', ', 0.25)')
        
        return {
          backgroundImage: `linear-gradient(to right, ${rgba1}, ${rgba2}, ${rgba3})`,
          backgroundSize: '200% 100%'
        }
      }
    }
    return {}
  }

  // Get shadow effects for different states
  const getShadowEffect = () => {
    if (isExtended) {
      return 'shadow-[0_0_15px_rgba(0,0,0,0.08)] dark:shadow-[0_0_15px_rgba(255,255,255,0.08)]'
    }
    if (isFocused) {
      return 'shadow-[0_0_0_1px_rgba(0,0,0,0.1)] dark:shadow-[0_0_0_1px_rgba(255,255,255,0.2)]'
    }
    return ''
  }

  return (
    <div 
      id="urlbar"
      className={cn(
        "flex items-center w-full transition-all duration-300 ease-in-out",
        singleToolbar ? "h-10" : "h-8"
      )}
      role="search" 
      aria-label={t('words.search')}
    >
      <div 
        className={cn(
          "grid grid-cols-[24px_1fr] items-center gap-1.5 h-full w-full rounded-[8px] px-2 transition-all duration-300 ease-in-out",
          !isExtended && "bg-[rgba(0,0,0,0.06)] dark:bg-[rgba(255,255,255,0.08)]",
          getShadowEffect()
        )}
        style={{
          ...getBackgroundStyle(),
          animation: isExtended ? 'loadingGradient 1.8s ease-in-out infinite' : 'none'
        }}
      >
        <span 
          className={cn(
            "inline-flex items-center justify-center text-current opacity-90 transition-all duration-300 ease-in-out"
          )} 
          aria-hidden
        >
          <SearchIcon 
            className="[&_*[stroke='context-fill']]:stroke-current [&_*[fill='context-fill']]:fill-current [&_*[stroke-opacity='context-fill-opacity']]:stroke-opacity-100" 
            width={16} 
            height={16} 
          />
        </span>
        
        <input 
          className="h-full w-full border-none outline-none rounded-[8px] bg-transparent text-text px-0.5 placeholder:text-text-tertiary transition-all duration-200" 
          placeholder={t('words.search')}
          value={searchTerm}
          onChange={(e) => setSearchTerm(e.target.value)}
          onKeyDown={handleKeyDown}
          onFocus={() => setIsFocused(true)}
          onBlur={() => setIsFocused(false)}
        />
        
        {/* Loading indicator */}
        {loadingState === 'loading' && (
          <div className="absolute bottom-0 left-0 h-0.5 bg-blue-500 rounded-b-[8px] transition-all duration-300 ease-in-out animate-pulse" />
        )}
        
        {/* Status messages */}
        {loadingState === 'loading' && (
          <div className="absolute -top-6 left-0 text-xs text-blue-500 animate-pulse">
            {t('words.searching')}...
          </div>
        )}
        {loadingState === 'success' && (
          <div className="absolute -top-6 left-0 text-xs text-green-500">
            {t('words.search-success')}
          </div>
        )}
        {loadingState === 'error' && (
          <div className="absolute -top-6 left-0 text-xs text-red-500">
            {t('words.search-failed')}
          </div>
        )}
      </div>

      <style>{`
        @keyframes loadingGradient {
          0% { background-position: 0% 50%; }
          50% { background-position: 100% 50%; }
          100% { background-position: 0% 50%; }
        }
      `}</style>
    </div>
  )
}
