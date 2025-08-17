import { useEffect, useRef, useCallback } from "react";

/**
 * create a throttled version of a function, only execute once in the specified delay time
 */
export function useThrottle<T extends (...args: any[]) => any>(
	callback: T,
	delay: number,
): T {
	const callbackRef = useRef(callback);

	const timeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

	const inThrottleRef = useRef(false);

	useEffect(() => {
		callbackRef.current = callback;
	}, [callback]);

	useEffect(() => {
		return () => {
			if (timeoutRef.current) {
				clearTimeout(timeoutRef.current);
			}
		};
	}, []);

	const throttledCallback = useCallback(
		(...args: Parameters<T>) => {
			if (inThrottleRef.current) {
				return;
			}
			callbackRef.current(...args);
			inThrottleRef.current = true;
			timeoutRef.current = setTimeout(() => {
				inThrottleRef.current = false;
			}, delay);
		},
		[delay],
	);

	return throttledCallback as T;
}
